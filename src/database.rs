use crate::app::{ChatMessage, DigestItem, LongTermMemoryItem};
use rusqlite::{Connection, OptionalExtension, Result as SqliteResult, params};
use std::path::PathBuf;
use uuid::Uuid;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> SqliteResult<Self> {
        let db_path = Self::get_db_path();

        // Ensure the directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&db_path)?;
        let db = Database { conn };
        db.initialize_tables()?;
        Ok(db)
    }

    fn get_db_path() -> PathBuf {
        // Get app data directory
        let mut path = if cfg!(target_os = "windows") {
            std::env::var("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| std::env::current_dir().unwrap())
        } else if cfg!(target_os = "macos") {
            std::env::var("HOME")
                .map(|home| PathBuf::from(home).join("Library/Application Support"))
                .unwrap_or_else(|_| std::env::current_dir().unwrap())
        } else {
            // Linux and others
            std::env::var("XDG_DATA_HOME")
                .map(PathBuf::from)
                .or_else(|_| {
                    std::env::var("HOME").map(|home| PathBuf::from(home).join(".local/share"))
                })
                .unwrap_or_else(|_| std::env::current_dir().unwrap())
        };

        path.push("egui-chatbot");
        path.push("chat_data.db");
        path
    }

    fn initialize_tables(&self) -> SqliteResult<()> {
        // Create content_items table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS content_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                role_or_source TEXT NOT NULL,
                timestamp_unix INTEGER,
                timestamp_display TEXT,
                original_id TEXT UNIQUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create panel_associations table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS panel_associations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_id INTEGER NOT NULL,
                panel_type TEXT NOT NULL,
                added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (content_id) REFERENCES content_items(id),
                UNIQUE(content_id, panel_type)
            )",
            [],
        )?;

        // Create assistant_roles table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS assistant_roles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                role_name TEXT UNIQUE NOT NULL,
                display_name TEXT NOT NULL,
                description TEXT,
                is_active BOOLEAN DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create system_prompts table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS system_prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                role_id INTEGER NOT NULL,
                panel_type TEXT NOT NULL,
                prompt_text TEXT NOT NULL,
                is_active BOOLEAN DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (role_id) REFERENCES assistant_roles(id),
                UNIQUE(role_id, panel_type)
            )",
            [],
        )?;

        // Create indexes for better performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_content_original_id ON content_items(original_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_panel_associations_content_id ON panel_associations(content_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_panel_associations_panel_type ON panel_associations(panel_type)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_system_prompts_role_id ON system_prompts(role_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_system_prompts_panel_type ON system_prompts(panel_type)",
            [],
        )?;

        // Insert initial roles and prompts if they don't exist
        self.insert_initial_roles_and_prompts()?;

        Ok(())
    }

    pub fn save_content(
        &self,
        content: &str,
        role_or_source: &str,
        timestamp_unix: i64,
        timestamp_display: &str,
        panel_types: &[&str],
    ) -> SqliteResult<()> {
        let original_id = Uuid::new_v4().to_string();

        // First, check if identical content already exists
        let existing_content_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM content_items WHERE content = ? AND role_or_source = ?",
                params![content, role_or_source],
                |row| row.get(0),
            )
            .optional()?;

        let content_id = if let Some(id) = existing_content_id {
            // Content already exists, use existing ID
            id
        } else {
            // Insert new content
            self.conn.execute(
                "INSERT INTO content_items (content, role_or_source, timestamp_unix, timestamp_display, original_id)
                 VALUES (?, ?, ?, ?, ?)",
                params![content, role_or_source, timestamp_unix, timestamp_display, original_id],
            )?;
            self.conn.last_insert_rowid()
        };

        // Add panel associations (using INSERT OR IGNORE to handle duplicates)
        for panel_type in panel_types {
            self.conn.execute(
                "INSERT OR IGNORE INTO panel_associations (content_id, panel_type) VALUES (?, ?)",
                params![content_id, panel_type],
            )?;
        }

        Ok(())
    }

    pub fn load_chat_messages(&self) -> SqliteResult<Vec<ChatMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT ci.content, ci.role_or_source
             FROM content_items ci
             JOIN panel_associations pa ON ci.id = pa.content_id
             WHERE pa.panel_type = 'chat'
             ORDER BY ci.timestamp_unix ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ChatMessage {
                role: row.get(1)?,
                content: row.get(0)?,
            })
        })?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }

        Ok(messages)
    }

    pub fn load_digest_items(&self) -> SqliteResult<Vec<DigestItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT ci.original_id, ci.content, ci.role_or_source, ci.timestamp_display
             FROM content_items ci
             JOIN panel_associations pa ON ci.id = pa.content_id
             WHERE pa.panel_type = 'digest'
             ORDER BY ci.timestamp_unix ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DigestItem {
                id: row.get(0)?,
                content: row.get(1)?,
                source: row.get(2)?,
                timestamp: row.get(3)?,
                selected: false, // Default to unselected when loading
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }

        Ok(items)
    }

    pub fn load_longterm_memory_items(&self) -> SqliteResult<Vec<LongTermMemoryItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT ci.original_id, ci.content, ci.role_or_source, ci.timestamp_display
             FROM content_items ci
             JOIN panel_associations pa ON ci.id = pa.content_id
             WHERE pa.panel_type = 'longterm'
             ORDER BY ci.timestamp_unix ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(LongTermMemoryItem {
                id: row.get(0)?,
                content: row.get(1)?,
                source: row.get(2)?,
                timestamp: row.get(3)?,
                selected: false, // Default to unselected when loading
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }

        Ok(items)
    }

    pub fn get_database_stats(&self) -> SqliteResult<(usize, usize, usize, usize)> {
        let total_content: usize =
            self.conn
                .query_row("SELECT COUNT(*) FROM content_items", [], |row| row.get(0))?;

        let chat_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM panel_associations WHERE panel_type = 'chat'",
            [],
            |row| row.get(0),
        )?;

        let digest_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM panel_associations WHERE panel_type = 'digest'",
            [],
            |row| row.get(0),
        )?;

        let longterm_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM panel_associations WHERE panel_type = 'longterm'",
            [],
            |row| row.get(0),
        )?;

        Ok((total_content, chat_count, digest_count, longterm_count))
    }

    fn insert_initial_roles_and_prompts(&self) -> SqliteResult<()> {
        // Check if roles already exist
        let role_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM assistant_roles",
            [],
            |row| row.get(0),
        )?;

        if role_count > 0 {
            return Ok(()); // Already initialized
        }

        // Insert Contract Template Selection role
        self.conn.execute(
            "INSERT INTO assistant_roles (role_name, display_name, description) VALUES (?, ?, ?)",
            params![
                "contract_template_selection",
                "Contract Template Selection",
                "Specialized in legal document analysis and contract template recommendations"
            ],
        )?;

        let contract_role_id = self.conn.last_insert_rowid();

        // Insert Procurement Template Selection role
        self.conn.execute(
            "INSERT INTO assistant_roles (role_name, display_name, description) VALUES (?, ?, ?)",
            params![
                "procurement_template_selection",
                "Procurement Template Selection",
                "Specialized in procurement processes and vendor management guidance"
            ],
        )?;

        let procurement_role_id = self.conn.last_insert_rowid();

        // Insert system prompts for Contract Template Selection role
        self.conn.execute(
            "INSERT INTO system_prompts (role_id, panel_type, prompt_text) VALUES (?, ?, ?)",
            params![
                contract_role_id,
                "chat",
                "You are a legal expert specializing in contract template selection and analysis. Help users identify the most appropriate contract templates based on their specific needs, analyze contract clauses, and provide guidance on legal document requirements. Focus on contract types, legal compliance, and template recommendations."
            ],
        )?;

        self.conn.execute(
            "INSERT INTO system_prompts (role_id, panel_type, prompt_text) VALUES (?, ?, ?)",
            params![
                contract_role_id,
                "digest",
                "When summarizing contract-related content, focus on extracting key legal clauses, contract terms, obligations, rights, and template recommendations. Highlight important legal considerations, compliance requirements, and critical contract elements that need attention."
            ],
        )?;

        self.conn.execute(
            "INSERT INTO system_prompts (role_id, panel_type, prompt_text) VALUES (?, ?, ?)",
            params![
                contract_role_id,
                "memory",
                "When processing long-term memory for contracts, organize information by contract types, legal precedents, standard clauses, and template patterns. Maintain knowledge of legal requirements, compliance standards, and best practices for contract template selection and management."
            ],
        )?;

        // Insert system prompts for Procurement Template Selection role
        self.conn.execute(
            "INSERT INTO system_prompts (role_id, panel_type, prompt_text) VALUES (?, ?, ?)",
            params![
                procurement_role_id,
                "chat",
                "You are a procurement expert specializing in procurement template selection and vendor management. Help users choose appropriate procurement templates (RFP, RFQ, vendor agreements), guide them through procurement processes, and provide best practices for vendor selection and management."
            ],
        )?;

        self.conn.execute(
            "INSERT INTO system_prompts (role_id, panel_type, prompt_text) VALUES (?, ?, ?)",
            params![
                procurement_role_id,
                "digest",
                "When summarizing procurement-related content, focus on extracting vendor requirements, procurement specifications, evaluation criteria, and template recommendations. Highlight key procurement processes, vendor qualifications, and critical decision points."
            ],
        )?;

        self.conn.execute(
            "INSERT INTO system_prompts (role_id, panel_type, prompt_text) VALUES (?, ?, ?)",
            params![
                procurement_role_id,
                "memory",
                "When processing long-term memory for procurement, organize information by procurement types, vendor categories, evaluation methodologies, and template patterns. Maintain knowledge of procurement best practices, vendor management strategies, and template selection criteria."
            ],
        )?;

        Ok(())
    }

    pub fn get_assistant_roles(&self) -> SqliteResult<Vec<(i64, String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, role_name, display_name, description FROM assistant_roles WHERE is_active = 1 ORDER BY display_name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        let mut roles = Vec::new();
        for row in rows {
            roles.push(row?);
        }

        Ok(roles)
    }

    pub fn get_system_prompts_for_role(&self, role_id: i64) -> SqliteResult<std::collections::HashMap<String, String>> {
        let mut stmt = self.conn.prepare(
            "SELECT panel_type, prompt_text FROM system_prompts WHERE role_id = ? AND is_active = 1"
        )?;

        let rows = stmt.query_map([role_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
            ))
        })?;

        let mut prompts = std::collections::HashMap::new();
        for row in rows {
            let (panel_type, prompt_text) = row?;
            prompts.insert(panel_type, prompt_text);
        }

        Ok(prompts)
    }
}
