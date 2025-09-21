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
            info_text: "DeepSeek Chat API Integration\nModel: deepseek-chat\nStreaming: Enabled\n中文支持: 已启用 (Chinese Support: Enabled)".to_owned(),

            // API configuration
            api_base_url: "https://api.deepseek.com".to_owned(),
            api_key: "sk-35177c12d8bc4125a78c3255072943fe".to_owned(), // Replace with actual API key
            model: "deepseek-chat".to_owned(),

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
            // Try macOS fonts
            for font_path in [
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
                "/Library/Fonts/Arial Unicode.ttf",
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

        // Send to API using the same chat API logic
        self.send_to_api(ctx);
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

        // Send to API using the same chat API logic
        self.send_to_api(ctx);
    }

    fn send_to_api(&mut self, ctx: &egui::Context) {
        let api_base_url = self.api_base_url.clone();
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let messages = self.chat_messages.clone();
        let ctx_clone = ctx.clone();

        let (tx, rx) = mpsc::channel();
        self.streaming_receiver = Some(rx);

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let api_url = format!("{}/chat/completions", api_base_url);

            let mut api_messages = Vec::new();
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
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });
        // BOTTOM PANEL: Chat Input (spans entire window width)
        // egui::CentralPanel::default().show(ctx, |ui| {

        egui::TopBottomPanel::bottom("chat_input_panel")
            .min_height(60.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
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
            });

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
