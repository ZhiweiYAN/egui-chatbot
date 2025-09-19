use std::sync::mpsc;
use futures::StreamExt;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

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
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World! to".to_owned(),
            value: 2.7,
            
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
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
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

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");


            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            // Chat history display
            ui.group(|ui| {
                ui.heading("Chat History");

                let scroll_area = egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .auto_shrink([false, false])
                    .stick_to_bottom(true);

                scroll_area.show(ui, |ui| {
                    if self.chat_messages.is_empty() {
                        ui.colored_label(egui::Color32::GRAY, "开始对话... (Start a conversation...)");
                    } else {
                        for (i, message) in self.chat_messages.iter().enumerate() {
                            if message.role == "user" {
                                ui.horizontal(|ui| {
                                    ui.colored_label(egui::Color32::LIGHT_BLUE, "You:");
                                    ui.label(&message.content);
                                });
                            } else if message.role == "assistant" {
                                if i == self.chat_messages.len() - 1 && self.is_waiting_response {
                                    // Show streaming response for the last assistant message
                                    if !self.current_response.is_empty() {
                                        ui.colored_label(egui::Color32::LIGHT_GREEN, &self.current_response);
                                    } else {
                                        ui.colored_label(egui::Color32::YELLOW, "🤖 typing...");
                                    }
                                } else {
                                    ui.colored_label(egui::Color32::LIGHT_GREEN, &message.content);
                                }
                                // Add spacing after assistant response (end of conversation turn)
                                ui.add_space(8.0);
                            }
                        }
                    }

                    if let Some(error) = &self.last_error {
                        ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                    }
                });
            });

            ui.separator();

            // Chat input at the bottom
            ui.heading("Send Message");

            ui.horizontal(|ui| {
                ui.label("Message:");
                let response = ui.text_edit_singleline(&mut self.chat_input);
                if self.should_focus_input {
                    response.request_focus();
                    self.should_focus_input = false; // Reset after requesting focus
                }

                let send_enabled = !self.chat_input.trim().is_empty() && !self.is_waiting_response;

                let send_clicked = ui.add_enabled(send_enabled, egui::Button::new("Send")).clicked();

                // Check for Enter key press globally when there's text to send
                let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.chat_input.trim().is_empty();

                if (send_clicked || enter_pressed) && send_enabled {

                    let user_message = self.chat_input.trim().to_string();
                    self.chat_input.clear();

                    // Add user message
                    self.chat_messages.push(ChatMessage {
                        role: "user".to_string(),
                        content: user_message.clone(),
                    });

                    // Add placeholder for assistant response
                    self.chat_messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: String::new(),
                    });

                    self.is_waiting_response = true;
                    self.current_response.clear(); // Clear previous response to show new one

                    // Send to API
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
                }
            });

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code. yzw"
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
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
