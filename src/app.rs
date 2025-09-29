use crate::database::Database;
use egui_commonmark::CommonMarkCache;
use futures::StreamExt;
use std::sync::mpsc;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct DigestItem {
    pub id: String,
    pub content: String,
    pub source: String, // "user" or "assistant"
    pub timestamp: String,
    #[serde(skip)]
    pub selected: bool,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LongTermMemoryItem {
    pub id: String,
    pub content: String,
    pub source: String, // "user" or "assistant"
    pub timestamp: String,
    #[serde(skip)]
    pub selected: bool,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    pub label: String,

    // Chat interface
    pub chat_input: String,
    pub chat_messages: Vec<ChatMessage>,

    // Information display
    pub info_text: String,

    // API configuration
    #[serde(skip)]
    pub api_base_url: String,
    #[serde(skip)]
    pub api_key: String,
    #[serde(skip)]
    pub model: String,

    // Streaming state
    #[serde(skip)]
    pub streaming_receiver: Option<mpsc::Receiver<String>>,
    #[serde(skip)]
    pub is_waiting_response: bool,
    #[serde(skip)]
    pub last_error: Option<String>,
    #[serde(skip)]
    pub current_response: String,
    #[serde(skip)]
    pub should_focus_input: bool,
    #[serde(skip)]
    pub should_scroll_chat: bool,

    // Digest functionality
    pub digest_items: Vec<DigestItem>,
    #[serde(skip)]
    pub selected_text: String,
    #[serde(skip)]
    pub digest_search: String,
    #[serde(skip)]
    pub chat_search: String,

    // Long term memory functionality
    pub long_term_memory_items: Vec<LongTermMemoryItem>,
    #[serde(skip)]
    pub memory_search: String,

    // Markdown cache for digest panel
    #[serde(skip)]
    pub markdown_cache: CommonMarkCache,

    // Database connection
    #[serde(skip)]
    pub database: Option<Database>,

    // Settings window
    #[serde(skip)]
    pub show_settings: bool,
    #[serde(skip)]
    pub temp_api_base_url: String,
    #[serde(skip)]
    pub temp_api_key: String,
    #[serde(skip)]
    pub temp_model: String,

    // Assistant role management
    #[serde(skip)]
    pub current_assistant_role_id: Option<i64>,
    #[serde(skip)]
    pub temp_assistant_role_id: Option<i64>,
    #[serde(skip)]
    pub available_roles: Vec<(i64, String, String, String)>, // (id, role_name, display_name, description)
    #[serde(skip)]
    pub current_system_prompts: std::collections::HashMap<String, String>, // panel_type -> prompt_text
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World! to".to_owned(),

            // Chat interface
            chat_input: String::new(),
            chat_messages: Vec::new(),

            // Information display
            info_text: "DeepSeek Chat API Integration\nModel: deepseek-chat\nStreaming: Enabled\n中文支持: 已启用 (Chinese Support: Enabled)\n测试字符: 杂 (Test character: 杂)".to_owned(),

            // API configuration from environment variables
            api_base_url: std::env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com".to_string()),
            api_key: std::env::var("LLM_API_KEY")
                .unwrap_or_else(|_| "".to_string()),
            model: std::env::var("LLM_MODEL")
                .unwrap_or_else(|_| "deepseek-chat".to_string()),

            // Streaming state
            streaming_receiver: None,
            is_waiting_response: false,
            last_error: None,
            current_response: String::new(),
            should_focus_input: false,
            should_scroll_chat: false,

            // Digest functionality
            digest_items: Vec::new(),
            selected_text: String::new(),
            digest_search: String::new(),
            chat_search: String::new(),

            // Long term memory functionality
            long_term_memory_items: Vec::new(),
            memory_search: String::new(),

            // Markdown cache for digest panel
            markdown_cache: CommonMarkCache::default(),

            // Database connection
            database: None,

            // Settings window
            show_settings: false,
            temp_api_base_url: std::env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com".to_string()),
            temp_api_key: std::env::var("LLM_API_KEY")
                .unwrap_or_else(|_| "".to_string()),
            temp_model: std::env::var("LLM_MODEL")
                .unwrap_or_else(|_| "deepseek-chat".to_string()),

            // Assistant role management
            current_assistant_role_id: None,
            temp_assistant_role_id: None,
            available_roles: Vec::new(),
            current_system_prompts: std::collections::HashMap::new(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Configure fonts to support Chinese characters
        Self::configure_fonts(&cc.egui_ctx);

        // Initialize database
        let database = match Database::new() {
            Ok(db) => Some(db),
            Err(e) => {
                log::error!("Failed to initialize database: {}", e);
                None
            }
        };

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let mut app: TemplateApp = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        // Set the database connection
        app.database = database;

        // Load assistant roles and set default role
        app.load_assistant_roles();

        app
    }

    fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // Try to load system fonts that support Chinese characters
        // Priority order: Microsoft YaHei, SimHei, Noto Sans CJK, fallback to built-in fonts

        // On Windows, try to load Microsoft YaHei
        if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
            fonts.font_data.insert(
                "Microsoft YaHei".to_owned(),
                egui::FontData::from_owned(font_data).into(),
            );

            // Set Microsoft YaHei as the primary font
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "Microsoft YaHei".to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "Microsoft YaHei".to_owned());
        }
        // Fallback: try to load other system Chinese fonts
        else {
            // Try to load PingFang.ttc - it should work with the entire collection
            if let Ok(font_data) = std::fs::read("/System/Library/Fonts/PingFang.ttc") {
                // Load the entire PingFang TrueType Collection as PingFangSC
                let egui_font_data = egui::FontData::from_owned(font_data);

                fonts.font_data.insert(
                    "PingFangSC".to_string(),
                    egui_font_data.into(),
                );

                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "PingFangSC".to_string());

                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "PingFangSC".to_string());

                log::info!("Loaded PingFangSC from PingFang.ttc");
            } else {
                log::warn!("Could not load PingFang.ttc");
            }

            // Try Linux fonts
            for font_path in [
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            ] {
                if let Ok(font_data) = std::fs::read(font_path) {
                    let font_name = format!("SystemFont{}", fonts.font_data.len());
                    fonts.font_data.insert(
                        font_name.clone(),
                        egui::FontData::from_owned(font_data).into(),
                    );

                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, font_name.clone());
                    break;
                }
            }
        }

        ctx.set_fonts(fonts);
    }

    pub fn render_highlighted_text(&self, ui: &mut egui::Ui, text: &str, search_term: &str) {
        if search_term.is_empty() {
            ui.label(text);
            return;
        }

        let search_lower = search_term.to_lowercase();
        let text_lower = text.to_lowercase();

        // Simple and safe approach: work entirely with characters
        let text_chars: Vec<char> = text.chars().collect();
        let text_lower_chars: Vec<char> = text_lower.chars().collect();
        let search_chars: Vec<char> = search_lower.chars().collect();

        if search_chars.is_empty() {
            ui.label(text);
            return;
        }

        // Find all character-based matches
        let mut matches = Vec::new();
        let mut start_idx = 0;

        while start_idx <= text_lower_chars.len().saturating_sub(search_chars.len()) {
            // Check if we have a match at this position
            let mut is_match = true;
            for (i, &search_char) in search_chars.iter().enumerate() {
                if start_idx + i >= text_lower_chars.len() || text_lower_chars[start_idx + i] != search_char {
                    is_match = false;
                    break;
                }
            }

            if is_match {
                matches.push((start_idx, start_idx + search_chars.len()));
                start_idx += 1; // Move by 1 to find overlapping matches
            } else {
                start_idx += 1;
            }
        }

        if matches.is_empty() {
            ui.label(text);
            return;
        }

        // Render text with highlights using character positions
        ui.horizontal_wrapped(|ui| {
            let mut last_pos = 0;

            for (start, end) in matches {
                // Render text before match
                if start > last_pos {
                    let before_text: String = text_chars[last_pos..start].iter().collect();
                    if !before_text.is_empty() {
                        ui.label(before_text);
                    }
                }

                // Render highlighted match
                if end <= text_chars.len() {
                    let match_text: String = text_chars[start..end].iter().collect();
                    if !match_text.is_empty() {
                        ui.colored_label(egui::Color32::DARK_RED, match_text);
                    }
                }

                last_pos = end;
            }

            // Render remaining text
            if last_pos < text_chars.len() {
                let remaining_text: String = text_chars[last_pos..].iter().collect();
                if !remaining_text.is_empty() {
                    ui.label(remaining_text);
                }
            }
        });
    }

    fn load_assistant_roles(&mut self) {
        if let Some(ref db) = self.database {
            // Load available roles
            match db.get_assistant_roles() {
                Ok(roles) => {
                    self.available_roles = roles;
                    // Set default role to the first one if no role is selected
                    if self.current_assistant_role_id.is_none() && !self.available_roles.is_empty() {
                        self.current_assistant_role_id = Some(self.available_roles[0].0);
                        self.temp_assistant_role_id = Some(self.available_roles[0].0);
                        self.load_system_prompts_for_current_role();
                    }
                }
                Err(e) => {
                    log::error!("Failed to load assistant roles: {}", e);
                }
            }
        }
    }

    fn load_system_prompts_for_current_role(&mut self) {
        if let (Some(role_id), Some(db)) = (self.current_assistant_role_id, &self.database) {
            match db.get_system_prompts_for_role(role_id) {
                Ok(prompts) => {
                    self.current_system_prompts = prompts;
                }
                Err(e) => {
                    log::error!("Failed to load system prompts for role {}: {}", role_id, e);
                }
            }
        }
    }

    pub fn add_to_digest(&mut self, content: String, source: String) {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = format!("{}_{}", source, timestamp);
        let formatted_time = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now())
            .format("%H:%M")
            .to_string();

        let digest_item = DigestItem {
            id,
            content: content.clone(),
            source: source.clone(),
            timestamp: formatted_time.clone(),
            selected: true, // Default to selected when adding new items
        };

        self.digest_items.push(digest_item);

        // Auto-save to database
        if let Some(ref db) = self.database {
            if let Err(e) = db.save_content(
                &content,
                &source,
                timestamp as i64,
                &formatted_time,
                &["digest"],
            ) {
                log::error!("Failed to save digest item to database: {}", e);
            }
        }
    }

    pub fn add_to_long_term_memory(&mut self, content: String, source: String) {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = format!("{}_{}", source, timestamp);
        let formatted_time = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now())
            .format("%H:%M")
            .to_string();

        let memory_item = LongTermMemoryItem {
            id,
            content: content.clone(),
            source: source.clone(),
            timestamp: formatted_time.clone(),
            selected: true, // Default to selected when adding new items
        };

        self.long_term_memory_items.push(memory_item);

        // Auto-save to database
        if let Some(ref db) = self.database {
            if let Err(e) = db.save_content(
                &content,
                &source,
                timestamp as i64,
                &formatted_time,
                &["longterm"],
            ) {
                log::error!("Failed to save longterm memory item to database: {}", e);
            }
        }
    }

    pub fn save_chat_message_to_db(&self, message: &ChatMessage) {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let formatted_time = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now())
            .format("%H:%M")
            .to_string();

        // Auto-save to database
        if let Some(ref db) = self.database {
            if let Err(e) = db.save_content(
                &message.content,
                &message.role,
                timestamp as i64,
                &formatted_time,
                &["chat"],
            ) {
                log::error!("Failed to save chat message to database: {}", e);
            }
        }
    }

    pub fn load_data_from_database(&mut self) {
        if let Some(ref db) = self.database {
            // Load chat messages
            match db.load_chat_messages() {
                Ok(messages) => {
                    self.chat_messages = messages;
                    log::info!(
                        "Loaded {} chat messages from database",
                        self.chat_messages.len()
                    );
                }
                Err(e) => {
                    log::error!("Failed to load chat messages from database: {}", e);
                }
            }

            // Load digest items
            match db.load_digest_items() {
                Ok(items) => {
                    self.digest_items = items;
                    log::info!(
                        "Loaded {} digest items from database",
                        self.digest_items.len()
                    );
                }
                Err(e) => {
                    log::error!("Failed to load digest items from database: {}", e);
                }
            }

            // Load long-term memory items
            match db.load_longterm_memory_items() {
                Ok(items) => {
                    self.long_term_memory_items = items;
                    log::info!(
                        "Loaded {} long-term memory items from database",
                        self.long_term_memory_items.len()
                    );
                }
                Err(e) => {
                    log::error!("Failed to load long-term memory items from database: {}", e);
                }
            }

            // Get database stats for info display
            match db.get_database_stats() {
                Ok((total_content, chat_count, digest_count, longterm_count)) => {
                    self.info_text = format!(
                        "Database loaded successfully!\nTotal unique content items: {}\nChat messages: {}\nDigest items: {}\nLong-term memory items: {}",
                        total_content, chat_count, digest_count, longterm_count
                    );
                }
                Err(e) => {
                    log::error!("Failed to get database stats: {}", e);
                }
            }
        } else {
            self.info_text = "Database not available. Cannot load data.".to_string();
            log::error!("Database not initialized. Cannot load data.");
        }
    }

    pub fn export_digest_items(&self) -> String {
        let mut export_text = String::new();
        export_text.push_str("# Digested Content Export\n\n");

        for (i, item) in self.digest_items.iter().enumerate() {
            export_text.push_str(&format!(
                "## Item {} - {} ({})\n\n",
                i + 1,
                if item.source == "user" {
                    "You"
                } else {
                    "Assistant"
                },
                item.timestamp
            ));
            export_text.push_str(&item.content);
            export_text.push_str("\n\n");
            export_text.push_str("---\n\n");
        }

        export_text.push_str(&format!("*Exported {} items*", self.digest_items.len()));
        export_text
    }

    pub fn export_memory_items(&self) -> String {
        let mut export_text = String::new();
        export_text.push_str("# Long Term Memory Export\n\n");

        for (i, item) in self.long_term_memory_items.iter().enumerate() {
            export_text.push_str(&format!(
                "## Item {} - {} ({})\n\n",
                i + 1,
                if item.source == "user" {
                    "You"
                } else {
                    "Assistant"
                },
                item.timestamp
            ));
            export_text.push_str(&item.content);
            export_text.push_str("\n\n");
            export_text.push_str("---\n\n");
        }

        export_text.push_str(&format!(
            "*Exported {} items*",
            self.long_term_memory_items.len()
        ));
        export_text
    }

    pub fn start_digest_summary_generation(&mut self, ctx: &egui::Context) {
        let selected_items: Vec<&DigestItem> = self
            .digest_items
            .iter()
            .filter(|item| item.selected)
            .collect();

        if selected_items.is_empty() || self.is_waiting_response {
            return;
        }

        // Prepare digest content for summarization
        let mut content_to_summarize = String::new();
        content_to_summarize
            .push_str("Please provide a comprehensive summary of the following digest items:\n\n");

        for (i, item) in selected_items.iter().enumerate() {
            content_to_summarize.push_str(&format!(
                "{}. {} ({}):\n{}\n\n",
                i + 1,
                if item.source == "user" {
                    "User"
                } else {
                    "Assistant"
                },
                item.timestamp,
                item.content
            ));
        }

        content_to_summarize.push_str("Please provide a clear, structured summary that captures the key points, main topics discussed, and important conclusions from the above content.");

        // Add user message for summary request to chat history
        self.chat_messages.push(ChatMessage {
            role: "user".to_string(),
            content: content_to_summarize.clone(),
        });

        // Add placeholder for assistant response
        self.chat_messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: String::new(),
        });

        self.is_waiting_response = true;
        self.current_response.clear();

        // Send ONLY the summary request to API (not full chat history)
        self.send_summary_to_api("digest", content_to_summarize, ctx);
    }

    pub fn start_memory_summary_generation(&mut self, ctx: &egui::Context) {
        let selected_items: Vec<&LongTermMemoryItem> = self
            .long_term_memory_items
            .iter()
            .filter(|item| item.selected)
            .collect();

        if selected_items.is_empty() || self.is_waiting_response {
            return;
        }

        // Prepare memory content for summarization
        let mut content_to_summarize = String::new();
        content_to_summarize.push_str(
            "Please provide a comprehensive summary of the following long term memory items:\n\n",
        );

        for (i, item) in selected_items.iter().enumerate() {
            content_to_summarize.push_str(&format!(
                "{}. {} ({}):\n{}\n\n",
                i + 1,
                if item.source == "user" {
                    "User"
                } else {
                    "Assistant"
                },
                item.timestamp,
                item.content
            ));
        }

        content_to_summarize.push_str("Please provide a clear, structured summary that captures the key points, main topics discussed, and important conclusions from the above content.");

        // Add user message for summary request
        self.chat_messages.push(ChatMessage {
            role: "user".to_string(),
            content: content_to_summarize.clone(),
        });

        // Add placeholder for assistant response
        self.chat_messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: String::new(),
        });

        self.is_waiting_response = true;
        self.current_response.clear();

        // Send ONLY the summary request to API (not full chat history)
        self.send_summary_to_api("memory", content_to_summarize, ctx);
    }

    fn send_to_api(&mut self, ctx: &egui::Context) {
        self.send_to_api_with_panel("chat", ctx);
    }

    fn send_summary_to_api(&mut self, panel_type: &str, summary_content: String, ctx: &egui::Context) {
        let api_base_url = self.api_base_url.clone();
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let system_prompt = self.current_system_prompts.get(panel_type).cloned();
        let ctx_clone = ctx.clone();

        let (tx, rx) = mpsc::channel();
        self.streaming_receiver = Some(rx);

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let api_url = format!("{}/chat/completions", api_base_url);

            let mut api_messages = Vec::new();

            // Add system prompt if available
            if let Some(system_prompt) = system_prompt {
                api_messages.push(serde_json::json!({
                    "role": "system",
                    "content": system_prompt
                }));
            }

            // Add ONLY the summary request (no chat history)
            api_messages.push(serde_json::json!({
                "role": "user",
                "content": summary_content
            }));

            let payload = serde_json::json!({
                "model": model,
                "messages": api_messages,
                "stream": true,
                "temperature": 0.3
            });

            // Debug: Print the HTTP body being sent to LLM
            println!("🔍 DEBUG - Summary HTTP Body sent to LLM API:");
            println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "Failed to serialize payload".to_string()));
            println!("─────────────────────────────────────");

            match client
                .post(&api_url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let mut stream = resp.bytes_stream();
                        let mut buffer = String::new();

                        while let Some(chunk_result) = stream.next().await {
                            match chunk_result {
                                Ok(chunk) => {
                                    let chunk_str = String::from_utf8_lossy(&chunk);
                                    buffer.push_str(&chunk_str);

                                    // Process complete lines
                                    while let Some(line_end) = buffer.find('\n') {
                                        let line = buffer[..line_end].trim().to_string();
                                        buffer = buffer[line_end + 1..].to_string();

                                        if line.starts_with("data: ") {
                                            let data = &line[6..];
                                            if data == "[DONE]" {
                                                let _ = tx.send("__STREAM_END__".to_string());
                                                ctx_clone.request_repaint();
                                                return;
                                            }

                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                                if let Some(choices) = json["choices"].as_array() {
                                                    if let Some(choice) = choices.first() {
                                                        if let Some(delta) = choice["delta"].as_object() {
                                                            if let Some(content) = delta["content"].as_str() {
                                                                let _ = tx.send(content.to_string());
                                                                ctx_clone.request_repaint();
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    } else {
                        let error_msg = format!("HTTP error: {}", resp.status());
                        let _ = tx.send(error_msg);
                        ctx_clone.request_repaint();
                    }
                }
                Err(e) => {
                    let error_msg = format!("Connection error: {}", e);
                    let _ = tx.send(error_msg);
                    ctx_clone.request_repaint();
                }
            }
        });
    }

    fn send_to_api_with_panel(&mut self, panel_type: &str, ctx: &egui::Context) {
        let api_base_url = self.api_base_url.clone();
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let messages = self.chat_messages.clone();
        let system_prompt = self.current_system_prompts.get(panel_type).cloned();
        let ctx_clone = ctx.clone();

        let (tx, rx) = mpsc::channel();
        self.streaming_receiver = Some(rx);

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let api_url = format!("{}/chat/completions", api_base_url);

            let mut api_messages = Vec::new();

            // Add system prompt if available
            if let Some(system_prompt) = system_prompt {
                api_messages.push(serde_json::json!({
                    "role": "system",
                    "content": system_prompt
                }));
            }

            // Add user and assistant messages
            for msg in &messages {
                if !msg.content.is_empty() {
                    api_messages.push(serde_json::json!({
                        "role": msg.role,
                        "content": msg.content
                    }));
                }
            }

            let payload = serde_json::json!({
                "model": model,
                "messages": api_messages,
                "stream": true,
                "temperature": 0.3
            });

            // Debug: Print the HTTP body being sent to LLM
            println!("🔍 DEBUG - HTTP Body sent to LLM API:");
            println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "Failed to serialize payload".to_string()));
            println!("─────────────────────────────────────");

            match client
                .post(&api_url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let mut stream = resp.bytes_stream();
                        let mut buffer = String::new();

                        while let Some(chunk_result) = stream.next().await {
                            match chunk_result {
                                Ok(chunk) => {
                                    let chunk_str = String::from_utf8_lossy(&chunk);
                                    buffer.push_str(&chunk_str);

                                    let lines: Vec<String> =
                                        buffer.split('\n').map(|s| s.to_string()).collect();
                                    let processed_lines: Vec<String> =
                                        lines[..lines.len().saturating_sub(1)].to_vec();
                                    buffer = lines.last().cloned().unwrap_or_default();

                                    for line in processed_lines {
                                        if line.starts_with("data: ") {
                                            let data = &line[6..];
                                            if data == "[DONE]" {
                                                let _ = tx.send("__STREAM_END__".to_string());
                                                ctx_clone.request_repaint();
                                                return;
                                            }

                                            if let Ok(json) =
                                                serde_json::from_str::<serde_json::Value>(data)
                                            {
                                                if let Some(choices) = json["choices"].as_array() {
                                                    if let Some(choice) = choices.first() {
                                                        if let Some(delta) =
                                                            choice["delta"].as_object()
                                                        {
                                                            if let Some(content) =
                                                                delta["content"].as_str()
                                                            {
                                                                if let Err(_e) =
                                                                    tx.send(content.to_string())
                                                                {
                                                                } else {
                                                                }
                                                                ctx_clone.request_repaint();
                                                            }
                                                        }
                                                        
                                                    }
                                                    
                                                    
                                                }
                                                
                                            }
                                        }
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    } else {
                        let status = resp.status();
                        let error_body = match resp.text().await {
                            Ok(body) => body,
                            Err(_) => "Unknown error".to_string(),
                        };
                        let _ = tx.send(format!("Error: HTTP {} - {}", status, error_body));
                        ctx_clone.request_repaint();
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("Connection error: {}", e));
                    ctx_clone.request_repaint();
                }
            }
        });
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle streaming responses
        if let Some(receiver) = &self.streaming_receiver {
            // Request frequent repaints while streaming
            ctx.request_repaint();

            // Process all available messages in this update cycle
            let mut received_any = false;
            loop {
                match receiver.try_recv() {
                    Ok(content) => {
                        received_any = true;
                        if content == "__STREAM_END__" {
                            // Stream finished - add final response to chat history
                            if !self.current_response.is_empty() {
                                // Find the last assistant message and update it
                                if let Some(last_msg) = self.chat_messages.last_mut() {
                                    if last_msg.role == "assistant" {
                                        last_msg.content = self.current_response.clone();
                                        // Save assistant response to database
                                        let msg_to_save = last_msg.clone();
                                        self.save_chat_message_to_db(&msg_to_save);
                                    }
                                }
                            }
                            self.streaming_receiver = None;
                            self.is_waiting_response = false;
                            self.last_error = None;
                            self.should_focus_input = true; // Request focus after response completes
                            // Don't clear current_response - keep it visible
                            break;
                        } else if content.is_empty() {
                            // Empty content can be part of streaming, don't close
                            continue;
                        } else if content.starts_with("Error:")
                            || content.starts_with("Connection error:")
                        {
                            // Handle errors
                            self.last_error = Some(content.clone());
                            self.current_response = format!("❌ {}", content);
                            self.streaming_receiver = None;
                            self.is_waiting_response = false;
                            break;
                        } else {
                            // Add content to current streaming response
                            self.current_response.push_str(&content);
                            self.last_error = None;
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.streaming_receiver = None;
                        self.is_waiting_response = false;
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        break;
                    }
                }
            }

            if received_any {
                ctx.request_repaint();
            }
        }

        // Top panel with menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Load from DB").clicked() {
                            self.load_data_from_database();
                        }
                        ui.separator();
                        if ui.button("Settings").clicked() {
                            self.show_settings = true;
                        }
                        ui.separator();
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        });
        // BOTTOM PANEL: Chat Input (spans entire window width)
        // egui::CentralPanel::default().show(ctx, |ui| {

        egui::TopBottomPanel::bottom("chat_input_panel")
            .min_height(60.0)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let input_response = ui.add_sized(
                        [ui.available_width() - 70.0, 25.0],
                        egui::TextEdit::singleline(&mut self.chat_input)
                            .hint_text("Type your message... (输入你的消息...)")
                            .font(egui::TextStyle::Body),
                    );

                    // Handle Enter key press and focus requests
                    if input_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        if !self.chat_input.trim().is_empty() && !self.is_waiting_response {
                            // Send message logic here
                            let user_message = ChatMessage {
                                role: "user".to_string(),
                                content: self.chat_input.clone(),
                            };
                            self.save_chat_message_to_db(&user_message);
                            self.chat_messages.push(user_message);

                            // Add placeholder for assistant response
                            self.chat_messages.push(ChatMessage {
                                role: "assistant".to_string(),
                                content: String::new(),
                            });

                            self.is_waiting_response = true;
                            self.current_response.clear();

                            self.send_to_api(ctx);
                            self.chat_input.clear();
                            self.should_focus_input = true;
                            self.should_scroll_chat = true;
                        }
                    }

                    // Handle focus requests
                    if self.should_focus_input {
                        input_response.request_focus();
                        self.should_focus_input = false;
                    }

                    let send_enabled =
                        !self.chat_input.trim().is_empty() && !self.is_waiting_response;
                    if ui
                        .add_enabled(send_enabled, egui::Button::new("Send"))
                        .clicked()
                    {
                        if !self.chat_input.trim().is_empty() && !self.is_waiting_response {
                            // Same send logic as Enter key
                            let user_message = ChatMessage {
                                role: "user".to_string(),
                                content: self.chat_input.clone(),
                            };
                            self.save_chat_message_to_db(&user_message);
                            self.chat_messages.push(user_message);

                            // Add placeholder for assistant response
                            self.chat_messages.push(ChatMessage {
                                role: "assistant".to_string(),
                                content: String::new(),
                            });

                            self.is_waiting_response = true;
                            self.current_response.clear();

                            self.send_to_api(ctx);
                            self.chat_input.clear();
                            self.should_focus_input = true;
                            self.should_scroll_chat = true;
                        }
                    }
                });

                // Add role indicator with reload button
                ui.separator();
                ui.horizontal(|ui| {
                    if let Some(role_id) = self.current_assistant_role_id {
                        if let Some((_, _, display_name, _)) = self.available_roles
                            .iter()
                            .find(|(id, _, _, _)| *id == role_id) {
                            // Add background frame for better visibility
                            let frame = egui::Frame::new()
                                .fill(egui::Color32::from_rgb(230, 245, 255))
                                .corner_radius(6.0)
                                .inner_margin(egui::Margin::symmetric(8, 4));
                            frame.show(ui, |ui| {
                                ui.colored_label(egui::Color32::from_rgb(0, 102, 204), format!("👤 {}", display_name));
                            });
                        } else {
                            let frame = egui::Frame::new()
                                .fill(egui::Color32::from_rgb(255, 240, 240))
                                .corner_radius(6.0)
                                .inner_margin(egui::Margin::symmetric(8, 4));
                            frame.show(ui, |ui| {
                                ui.colored_label(egui::Color32::from_rgb(204, 0, 0), "👤 Unknown Role");
                            });
                        }

                        // Add reload button (enabled when role is selected)
                        if ui.small_button("🔄 Reload").on_hover_text("Reload system prompts from database").clicked() {
                            self.load_system_prompts_for_current_role();
                        }
                    } else {
                        let frame = egui::Frame::new()
                            .fill(egui::Color32::from_rgb(248, 248, 248))
                            .corner_radius(6.0)
                            .inner_margin(egui::Margin::symmetric(8, 4));
                        frame.show(ui, |ui| {
                            ui.colored_label(egui::Color32::from_rgb(128, 128, 128), "👤 No Role Selected");
                        });

                        // Add disabled reload button when no role selected
                        ui.add_enabled(false, egui::Button::new("🔄 Reload").small())
                            .on_hover_text("Select a role first to enable reload");
                    }
                });
            });

        // Show settings window if requested
        if self.show_settings {
            egui::Window::new("Settings")
                .collapsible(false)
                .resizable(true)
                .default_width(400.0)
                .show(ctx, |ui| {
                    ui.heading("LLM Configuration");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Base URL:");
                        ui.text_edit_singleline(&mut self.temp_api_base_url);
                    });

                    ui.horizontal(|ui| {
                        ui.label("API Key:");
                        ui.text_edit_singleline(&mut self.temp_api_key);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Model:");
                        ui.text_edit_singleline(&mut self.temp_model);
                    });

                    ui.separator();
                    ui.heading("Assistant Role");
                    ui.separator();

                    // Role selection dropdown
                    ui.horizontal(|ui| {
                        ui.label("Role:");

                        let current_role_name = if let Some(role_id) = self.temp_assistant_role_id {
                            self.available_roles
                                .iter()
                                .find(|(id, _, _, _)| *id == role_id)
                                .map(|(_, _, display_name, _)| display_name.clone())
                                .unwrap_or_else(|| "Unknown Role".to_string())
                        } else {
                            "No Role Selected".to_string()
                        };

                        egui::ComboBox::from_label("")
                            .selected_text(current_role_name)
                            .show_ui(ui, |ui| {
                                for (role_id, _role_name, display_name, description) in &self.available_roles {
                                    let is_selected = self.temp_assistant_role_id == Some(*role_id);
                                    let response = ui.selectable_label(is_selected, display_name);
                                    if response.clicked() {
                                        self.temp_assistant_role_id = Some(*role_id);
                                    }
                                    if response.hovered() {
                                        response.on_hover_text(description);
                                    }
                                }
                            });
                    });

                    // Show current role description if available
                    if let Some(role_id) = self.temp_assistant_role_id {
                        if let Some((_, _, _, description)) = self.available_roles
                            .iter()
                            .find(|(id, _, _, _)| *id == role_id) {
                            ui.colored_label(egui::Color32::GRAY, description);
                        }
                    }

                    ui.separator();
                    ui.heading("Database Information");
                    ui.separator();

                    // Display database path
                    ui.horizontal(|ui| {
                        ui.label("Database Path:");
                        let db_path = crate::database::Database::get_database_path();
                        ui.selectable_label(false, db_path.to_string_lossy().to_string())
                            .on_hover_text("Click to select and copy the database path");
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() {
                            // Apply settings
                            self.api_base_url = self.temp_api_base_url.clone();
                            self.api_key = self.temp_api_key.clone();
                            self.model = self.temp_model.clone();

                            // Apply role change
                            if self.current_assistant_role_id != self.temp_assistant_role_id {
                                self.current_assistant_role_id = self.temp_assistant_role_id;
                                self.load_system_prompts_for_current_role();
                            }

                            self.show_settings = false;
                        }
                        if ui.button("Cancel").clicked() {
                            // Reset temporary values to current values
                            self.temp_api_base_url = self.api_base_url.clone();
                            self.temp_api_key = self.api_key.clone();
                            self.temp_model = self.model.clone();
                            self.temp_assistant_role_id = self.current_assistant_role_id;
                            self.show_settings = false;
                        }
                    });
                });
        }

        // Render the three panels using the separate modules
        let (digest_actions, memory_actions_from_chat) = self.render_chat_panel(ctx);
        self.render_long_mem_panel(ctx);
        let memory_actions_from_digest = self.render_digest_panel(ctx);

        // Process all actions from both panels
        for (content, source) in digest_actions {
            self.add_to_digest(content, source);
        }
        for (content, source) in memory_actions_from_chat {
            self.add_to_long_term_memory(content, source);
        }
        for (content, source) in memory_actions_from_digest {
            self.add_to_long_term_memory(content, source);
        }
    }
}

// fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
//     ui.horizontal(|ui| {
//         ui.spacing_mut().item_spacing.x = 0.0;
//         ui.label("Powered by ");
//         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
//         ui.label(" and ");
//         ui.hyperlink_to(
//             "eframe",
//             "https://github.com/emilk/egui/tree/master/crates/eframe",
//         );
//         ui.label(".");
//     });
// }
