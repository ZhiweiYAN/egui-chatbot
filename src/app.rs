use std::sync::mpsc;
use futures::StreamExt;

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
    label: String,

    // Chat interface
    chat_input: String,
    chat_messages: Vec<ChatMessage>,

    // Information display
    info_text: String,

    // API configuration
    #[serde(skip)]
    api_base_url: String,
    #[serde(skip)]
    api_key: String,
    #[serde(skip)]
    model: String,

    // Streaming state
    #[serde(skip)]
    streaming_receiver: Option<mpsc::Receiver<String>>,
    #[serde(skip)]
    is_waiting_response: bool,
    #[serde(skip)]
    last_error: Option<String>,
    #[serde(skip)]
    current_response: String,
    #[serde(skip)]
    should_focus_input: bool,

    // Digest functionality
    digest_items: Vec<DigestItem>,
    #[serde(skip)]
    selected_text: String,
    #[serde(skip)]
    digest_search: String,
    #[serde(skip)]
    chat_search: String,

    // Long term memory functionality
    long_term_memory_items: Vec<LongTermMemoryItem>,
    #[serde(skip)]
    memory_search: String,
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

            // Digest functionality
            digest_items: Vec::new(),
            selected_text: String::new(),
            digest_search: String::new(),
            chat_search: String::new(),

            // Long term memory functionality
            long_term_memory_items: Vec::new(),
            memory_search: String::new(),
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

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
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

    fn add_to_digest(&mut self, content: String, source: String) {
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
            content,
            source,
            timestamp: formatted_time,
            selected: true, // Default to selected when adding new items
        };

        self.digest_items.push(digest_item);
    }

    fn add_to_long_term_memory(&mut self, content: String, source: String) {
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
            content,
            source,
            timestamp: formatted_time,
            selected: true, // Default to selected when adding new items
        };

        self.long_term_memory_items.push(memory_item);
    }

    fn export_digest_items(&self) -> String {
        let mut export_text = String::new();
        export_text.push_str("# Digested Content Export\n\n");

        for (i, item) in self.digest_items.iter().enumerate() {
            export_text.push_str(&format!("## Item {} - {} ({})\n\n",
                i + 1,
                if item.source == "user" { "You" } else { "Assistant" },
                item.timestamp
            ));
            export_text.push_str(&item.content);
            export_text.push_str("\n\n");
            export_text.push_str("---\n\n");
        }

        export_text.push_str(&format!("*Exported {} items*", self.digest_items.len()));
        export_text
    }

    fn export_memory_items(&self) -> String {
        let mut export_text = String::new();
        export_text.push_str("# Long Term Memory Export\n\n");

        for (i, item) in self.long_term_memory_items.iter().enumerate() {
            export_text.push_str(&format!("## Item {} - {} ({})\n\n",
                i + 1,
                if item.source == "user" { "You" } else { "Assistant" },
                item.timestamp
            ));
            export_text.push_str(&item.content);
            export_text.push_str("\n\n");
            export_text.push_str("---\n\n");
        }

        export_text.push_str(&format!("*Exported {} items*", self.long_term_memory_items.len()));
        export_text
    }

    fn start_summary_generation(&mut self, ctx: &egui::Context) {
        let selected_items: Vec<&DigestItem> = self.digest_items.iter().filter(|item| item.selected).collect();

        if selected_items.is_empty() || self.is_waiting_response {
            return;
        }

        // Prepare digest content for summarization
        let mut content_to_summarize = String::new();
        content_to_summarize.push_str("Please provide a comprehensive summary of the following digest items:\n\n");

        for (i, item) in selected_items.iter().enumerate() {
            content_to_summarize.push_str(&format!("{}. {} ({}):\n{}\n\n",
                i + 1,
                if item.source == "user" { "User" } else { "Assistant" },
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

                                    let lines: Vec<String> = buffer.split('\n').map(|s| s.to_string()).collect();
                                    let processed_lines: Vec<String> = lines[..lines.len().saturating_sub(1)].to_vec();
                                    buffer = lines.last().cloned().unwrap_or_default();

                                    for line in processed_lines {
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
                                                                if let Err(_e) = tx.send(content.to_string()) {
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

    fn start_memory_summary_generation(&mut self, ctx: &egui::Context) {
        let selected_items: Vec<&LongTermMemoryItem> = self.long_term_memory_items.iter().filter(|item| item.selected).collect();

        if selected_items.is_empty() || self.is_waiting_response {
            return;
        }

        // Prepare memory content for summarization
        let mut content_to_summarize = String::new();
        content_to_summarize.push_str("Please provide a comprehensive summary of the following long term memory items:\n\n");

        for (i, item) in selected_items.iter().enumerate() {
            content_to_summarize.push_str(&format!("{}. {} ({}):\n{}\n\n",
                i + 1,
                if item.source == "user" { "User" } else { "Assistant" },
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

                                    let lines: Vec<String> = buffer.split('\n').map(|s| s.to_string()).collect();
                                    let processed_lines: Vec<String> = lines[..lines.len().saturating_sub(1)].to_vec();
                                    buffer = lines.last().cloned().unwrap_or_default();

                                    for line in processed_lines {
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
                                                                if let Err(_e) = tx.send(content.to_string()) {
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
                        } else if content.starts_with("Error:") || content.starts_with("Connection error:") {
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
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        // LEFT PANEL: Chat History
        egui::SidePanel::left("chat_history")
            .default_width(400.0)
            .min_width(300.0)
            .max_width(600.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("💬 Chat History");

                // Search box
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut self.chat_search)
                        .on_hover_text("Search chat messages");
                    if ui.small_button("✖").on_hover_text("Clear search").clicked() {
                        self.chat_search.clear();
                    }
                });
                ui.separator();

                let scroll_area = egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true);

                let mut digest_actions: Vec<(String, String)> = Vec::new();
                let mut memory_actions: Vec<(String, String)> = Vec::new();

                scroll_area.show(ui, |ui| {
                    if self.chat_messages.is_empty() {
                        ui.colored_label(egui::Color32::GRAY, "开始对话... (Start a conversation...)");
                    } else {
                        let search_term = self.chat_search.to_lowercase();
                        let filtered_indices: Vec<usize> = self.chat_messages
                            .iter()
                            .enumerate()
                            .filter(|(_, message)| {
                                if search_term.is_empty() {
                                    true
                                } else {
                                    message.content.to_lowercase().contains(&search_term) ||
                                    message.role.to_lowercase().contains(&search_term)
                                }
                            })
                            .map(|(i, _)| i)
                            .collect();

                        if filtered_indices.is_empty() && !search_term.is_empty() {
                            ui.colored_label(egui::Color32::GRAY, "No messages match your search.");
                        } else {
                            for i in filtered_indices {
                                let message = &self.chat_messages[i];
                                if message.role == "user" {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.colored_label(egui::Color32::LIGHT_BLUE, "You:");
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("M Memory").clicked() {
                                                    memory_actions.push((message.content.clone(), message.role.clone()));
                                                }
                                                if ui.button("📋 Digest").clicked() {
                                                    digest_actions.push((message.content.clone(), message.role.clone()));
                                                }
                                            });
                                        });
                                        ui.label(&message.content);
                                    });
                                } else if message.role == "assistant" {
                                    if i == self.chat_messages.len() - 1 && self.is_waiting_response {
                                        // Show streaming response for the last assistant message
                                        if !self.current_response.is_empty() {
                                            ui.vertical(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if ui.small_button("M Memory").clicked() {
                                                            memory_actions.push((self.current_response.clone(), "assistant".to_string()));
                                                        }
                                                        if ui.button("📋 Digest").clicked() {
                                                            digest_actions.push((self.current_response.clone(), "assistant".to_string()));
                                                        }
                                                    });
                                                });
                                                ui.colored_label(egui::Color32::LIGHT_GREEN, &self.current_response);
                                            });
                                        } else {
                                            ui.colored_label(egui::Color32::YELLOW, "🤖 typing...");
                                        }
                                    } else {
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.small_button("M Memory").clicked() {
                                                        memory_actions.push((message.content.clone(), message.role.clone()));
                                                    }
                                                    if ui.button("📋 Digest").clicked() {
                                                        digest_actions.push((message.content.clone(), message.role.clone()));
                                                    }
                                                });
                                            });
                                            ui.label(&message.content);
                                        });
                                    }
                                    // Add spacing after assistant response (end of conversation turn)
                                    ui.add_space(8.0);
                                }
                            }
                        }
                    }

                    if let Some(error) = &self.last_error {
                        ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                    }
                });

                // Process digest and memory actions after the scroll area
                for (content, source) in digest_actions {
                    self.add_to_digest(content, source);
                }
                for (content, source) in memory_actions {
                    self.add_to_long_term_memory(content, source);
                }
            });

        // RIGHT PANEL: Long Term Memory
        egui::SidePanel::right("long_term_memory")
            .default_width(350.0)
            .min_width(250.0)
            .max_width(500.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("M Long Term Memory");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let selected_count = self.long_term_memory_items.iter().filter(|item| item.selected).count();
                        let summary_enabled = selected_count > 0 && !self.is_waiting_response;
                        let button_text = if self.is_waiting_response {
                            "🤖 Processing...".to_string()
                        } else if selected_count > 0 {
                            format!("📄 Summary ({})", selected_count)
                        } else {
                            "📄 Summary".to_string()
                        };

                        if ui.add_enabled(summary_enabled, egui::Button::new(button_text))
                            .on_hover_text("Generate a summary of selected memory items and show in chat")
                            .clicked()
                        {
                            self.start_memory_summary_generation(ui.ctx());
                        }
                    });
                });

                // Search box
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut self.memory_search)
                        .on_hover_text("Search memory items");
                    if ui.small_button("✖").on_hover_text("Clear search").clicked() {
                        self.memory_search.clear();
                    }
                });
                ui.separator();

                let mut item_to_delete: Option<usize> = None;

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if self.long_term_memory_items.is_empty() {
                            ui.colored_label(egui::Color32::GRAY, "No memory items yet.\nClick '🧠 Memory' on chat messages to store important content.");
                        } else {
                            let search_term = self.memory_search.to_lowercase();
                            let filtered_indices: Vec<usize> = self.long_term_memory_items
                                .iter()
                                .enumerate()
                                .filter(|(_, item)| {
                                    if search_term.is_empty() {
                                        true
                                    } else {
                                        item.content.to_lowercase().contains(&search_term) ||
                                        item.source.to_lowercase().contains(&search_term)
                                    }
                                })
                                .map(|(i, _)| i)
                                .collect();

                            if filtered_indices.is_empty() && !search_term.is_empty() {
                                ui.colored_label(egui::Color32::GRAY, "No items match your search.");
                            } else {
                                for i in filtered_indices {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut self.long_term_memory_items[i].selected, "");
                                        ui.colored_label(
                                            if self.long_term_memory_items[i].source == "user" { egui::Color32::LIGHT_BLUE } else { egui::Color32::LIGHT_GREEN },
                                            &format!("{}:", if self.long_term_memory_items[i].source == "user" { "You" } else { "🤖 Assistant" })
                                        );
                                        ui.label(&self.long_term_memory_items[i].timestamp);
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button("🗑").on_hover_text("Delete item").clicked() {
                                                item_to_delete = Some(i);
                                            }
                                            if ui.small_button("📋").on_hover_text("Copy to clipboard").clicked() {
                                                ui.ctx().copy_text(self.long_term_memory_items[i].content.clone());
                                            }
                                        });
                                    });
                                    ui.label(&self.long_term_memory_items[i].content);
                                    ui.add_space(5.0);
                                }
                            }
                        }
                    });

                if let Some(index) = item_to_delete {
                    self.long_term_memory_items.remove(index);
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Clear All").clicked() && !self.long_term_memory_items.is_empty() {
                        self.long_term_memory_items.clear();
                    }
                    if ui.button("Export All").clicked() && !self.long_term_memory_items.is_empty() {
                        let export_text = self.export_memory_items();
                        ui.ctx().copy_text(export_text);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("Items: {}", self.long_term_memory_items.len()));
                    });
                });
            });

        // CENTER PANEL: Digest Content
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📋 Digested Content");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let selected_count = self.digest_items.iter().filter(|item| item.selected).count();
                    let summary_enabled = selected_count > 0 && !self.is_waiting_response;
                    let button_text = if self.is_waiting_response {
                        "🤖 Processing...".to_string()
                    } else if selected_count > 0 {
                        format!("📄 Summary ({})", selected_count)
                    } else {
                        "📄 Summary".to_string()
                    };

                    if ui.add_enabled(summary_enabled, egui::Button::new(button_text))
                        .on_hover_text("Generate a summary of selected digest items and show in chat")
                        .clicked()
                    {
                        self.start_summary_generation(ui.ctx());
                    }
                });
            });

            // Search box
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut self.digest_search)
                    .on_hover_text("Search digest items");
                if ui.small_button("✖").on_hover_text("Clear search").clicked() {
                    self.digest_search.clear();
                }
            });
            ui.separator();

            let mut item_to_delete: Option<usize> = None;
            let mut memory_actions: Vec<(String, String)> = Vec::new();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if self.digest_items.is_empty() {
                        ui.colored_label(egui::Color32::GRAY, "No digest items yet.\nClick '📋 Digest' on chat messages to collect important content.");
                    } else {
                        let search_term = self.digest_search.to_lowercase();
                        let filtered_indices: Vec<usize> = self.digest_items
                            .iter()
                            .enumerate()
                            .filter(|(_, item)| {
                                if search_term.is_empty() {
                                    true
                                } else {
                                    item.content.to_lowercase().contains(&search_term) ||
                                    item.source.to_lowercase().contains(&search_term)
                                }
                            })
                            .map(|(i, _)| i)
                            .collect();

                        if filtered_indices.is_empty() && !search_term.is_empty() {
                            ui.colored_label(egui::Color32::GRAY, "No items match your search.");
                        } else {
                            for i in filtered_indices {
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut self.digest_items[i].selected, "");
                                    ui.colored_label(
                                        if self.digest_items[i].source == "user" { egui::Color32::LIGHT_BLUE } else { egui::Color32::LIGHT_GREEN },
                                        &format!("{}:", if self.digest_items[i].source == "user" { "You" } else { "🤖 Assistant" })
                                    );
                                    ui.label(&self.digest_items[i].timestamp);
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("🗑").on_hover_text("Delete item").clicked() {
                                            item_to_delete = Some(i);
                                        }
                                        if ui.small_button("M").on_hover_text("Copy to Long Term Memory").clicked() {
                                            memory_actions.push((self.digest_items[i].content.clone(), self.digest_items[i].source.clone()));
                                        }
                                        if ui.small_button("📋").on_hover_text("Copy to clipboard").clicked() {
                                            ui.ctx().copy_text(self.digest_items[i].content.clone());
                                        }
                                    });
                                });
                                ui.label(&self.digest_items[i].content);
                                ui.add_space(5.0);
                            }
                        }
                    }
                });

            // Process memory actions after the scroll area
            for (content, source) in memory_actions {
                self.add_to_long_term_memory(content, source);
            }

            if let Some(index) = item_to_delete {
                self.digest_items.remove(index);
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Clear All").clicked() && !self.digest_items.is_empty() {
                    self.digest_items.clear();
                }
                if ui.button("Export All").clicked() && !self.digest_items.is_empty() {
                    let export_text = self.export_digest_items();
                    ui.ctx().copy_text(export_text);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Items: {}", self.digest_items.len()));
                });
            });
        });

        // BOTTOM PANEL: Chat Input (spans entire window width)
        egui::TopBottomPanel::bottom("chat_input_panel")
            .min_height(60.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let input_response = ui.add_sized(
                        [ui.available_width() - 70.0, 25.0],
                        egui::TextEdit::singleline(&mut self.chat_input)
                            .hint_text("Type your message... (输入你的消息...)")
                            .font(egui::TextStyle::Body)
                    );

                    // Handle Enter key press and focus requests
                    if input_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !self.chat_input.trim().is_empty() && !self.is_waiting_response {
                            // Send message logic here
                            let user_message = ChatMessage {
                                role: "user".to_string(),
                                content: self.chat_input.clone(),
                            };
                            self.chat_messages.push(user_message);

                            // Add placeholder for assistant response
                            self.chat_messages.push(ChatMessage {
                                role: "assistant".to_string(),
                                content: String::new(),
                            });

                            self.is_waiting_response = true;
                            self.current_response.clear();

                            // Start API call
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
                                    "temperature": 0.7
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

                                                        let lines: Vec<String> = buffer.split('\n').map(|s| s.to_string()).collect();
                                                        let processed_lines: Vec<String> = lines[..lines.len().saturating_sub(1)].to_vec();
                                                        buffer = lines.last().cloned().unwrap_or_default();

                                                        for line in processed_lines {
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
                                                                                    if let Err(_e) = tx.send(content.to_string()) {
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

                            self.chat_input.clear();
                            self.should_focus_input = true;
                        }
                    }

                    // Handle focus requests
                    if self.should_focus_input {
                        input_response.request_focus();
                        self.should_focus_input = false;
                    }

                    let send_enabled = !self.chat_input.trim().is_empty() && !self.is_waiting_response;
                    if ui.add_enabled(send_enabled, egui::Button::new("Send")).clicked() {
                        if !self.chat_input.trim().is_empty() && !self.is_waiting_response {
                            // Same send logic as Enter key
                            let user_message = ChatMessage {
                                role: "user".to_string(),
                                content: self.chat_input.clone(),
                            };
                            self.chat_messages.push(user_message);

                            // Add placeholder for assistant response
                            self.chat_messages.push(ChatMessage {
                                role: "assistant".to_string(),
                                content: String::new(),
                            });

                            self.is_waiting_response = true;
                            self.current_response.clear();

                            // Start API call (same as above)
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
                                    "temperature": 0.7
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

                                                        let lines: Vec<String> = buffer.split('\n').map(|s| s.to_string()).collect();
                                                        let processed_lines: Vec<String> = lines[..lines.len().saturating_sub(1)].to_vec();
                                                        buffer = lines.last().cloned().unwrap_or_default();

                                                        for line in processed_lines {
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
                                                                                    if let Err(_e) = tx.send(content.to_string()) {
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

                            self.chat_input.clear();
                            self.should_focus_input = true;
                        }
                    }
                });
            });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}