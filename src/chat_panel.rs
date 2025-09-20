use crate::app::TemplateApp;

impl TemplateApp {
    pub fn render_chat_panel(&mut self, ctx: &egui::Context) -> (Vec<(String, String)>, Vec<(String, String)>) {
        let mut digest_actions = Vec::new();
        let mut memory_actions = Vec::new();

        egui::SidePanel::left("chat_history")
            .default_width(400.0)
            .min_width(300.0)
            .max_width(600.0)
            .resizable(true)
            .show(ctx, |ui| {
                // ui.add_space(6.0);
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

                let mut scroll_output = scroll_area.show(ui, |ui| {
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
                                                ui.colored_label(egui::Color32::DARK_GREEN, &self.current_response);
                                            });
                                        } else {
                                            ui.colored_label(egui::Color32::BROWN, "🤖 typing...");
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

                // Scroll to bottom when explicitly requested (Enter key or Send button)
                if self.should_scroll_chat {
                    scroll_output.state.offset.y = f32::INFINITY;
                    self.should_scroll_chat = false;
                }
            });

        (digest_actions, memory_actions)
    }
}