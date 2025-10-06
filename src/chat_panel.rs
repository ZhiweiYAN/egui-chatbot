use crate::app::TemplateApp;
use egui_commonmark::CommonMarkViewer;

type ActionList = Vec<(String, String)>;

impl TemplateApp {
    #[expect(clippy::too_many_lines)]
    pub fn render_chat_panel(
        &mut self,
        ctx: &egui::Context,
    ) -> (ActionList, ActionList) {
        let mut digest_actions = Vec::new();
        let mut memory_actions = Vec::new();
        let mut message_to_delete: Option<usize> = None;

        egui::SidePanel::left("chat_history")
            .default_width(400.0)
            .min_width(300.0)
            .max_width(500.0)
            .resizable(true)
            .show(ctx, |ui| {
                // ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.heading("💬 Chat History");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button("🗑 Clear")
                            .on_hover_text("Clear all chat messages")
                            .clicked()
                        {
                            // Clear chat panel associations from database (soft delete)
                            if let Some(ref db) = self.database {
                                if let Err(e) = db.clear_chat_panel_associations() {
                                    log::error!("Failed to clear chat panel associations: {e}");
                                    self.last_error = Some(format!("Database error: {e}"));
                                }
                            }

                            // Clear UI state
                            self.chat_messages.clear();
                            self.current_response.clear();
                            self.is_waiting_response = false;
                        }
                    });
                });

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
                        ui.colored_label(
                            egui::Color32::GRAY,
                            "开始对话... (Start a conversation...)",
                        );
                    } else {
                        let search_term = self.chat_search.to_lowercase();
                        let search_query = self.chat_search.clone(); // Keep original case for highlighting
                        let filtered_indices: Vec<usize> = self
                            .chat_messages
                            .iter()
                            .enumerate()
                            .filter(|(_, message)| {
                                if search_term.is_empty() {
                                    true
                                } else {
                                    message.content.to_lowercase().contains(&search_term)
                                        || message.role.to_lowercase().contains(&search_term)
                                }
                            })
                            .map(|(i, _)| i)
                            .collect();

                        if filtered_indices.is_empty() && !search_term.is_empty() {
                            ui.colored_label(egui::Color32::GRAY, "No messages match your search.");
                        } else {
                            for i in filtered_indices {
                                let message = &self.chat_messages[i];
                                let message_content = message.content.clone(); // Clone to avoid borrowing issues
                                let message_role = message.role.clone();

                                if message_role == "user" {
                                    ui.vertical(|ui| {
                                        ui.colored_label(egui::Color32::DARK_RED, "You:");
                                        // Add background frame for user messages
                                        let frame = egui::Frame::new()
                                            .fill(egui::Color32::from_rgb(238, 235, 226))
                                            .corner_radius(4.0)
                                            .inner_margin(8.0);
                                        frame.show(ui, |ui| {
                                            ui.set_max_width(ui.available_width());
                                            ui.style_mut().wrap_mode =
                                                Some(egui::TextWrapMode::Wrap);
                                            self.render_highlighted_text(
                                                ui,
                                                &message_content,
                                                &search_query,
                                            );
                                        });

                                        // Add buttons at the end of message
                                        ui.horizontal(|ui| {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui
                                                        .small_button(egui::RichText::new("🗑").color(egui::Color32::from_rgb(0x8E, 0x94, 0x9B)))
                                                        .on_hover_text("Delete message")
                                                        .clicked()
                                                    {
                                                        message_to_delete = Some(i);
                                                    }
                                                    if ui.small_button(egui::RichText::new("🗄 Memory").color(egui::Color32::from_rgb(0x8E, 0x94, 0x9B))).clicked() {
                                                        memory_actions.push((
                                                            message_content.clone(),
                                                            message_role.clone(),
                                                        ));
                                                    }
                                                    if ui.button(egui::RichText::new("📌 Digest").color(egui::Color32::from_rgb(0x8E, 0x94, 0x9B))).clicked() {
                                                        digest_actions.push((
                                                            message_content.clone(),
                                                            message_role.clone(),
                                                        ));
                                                    }
                                                },
                                            );
                                        });
                                    });
                                } else if message_role == "assistant" {
                                    if i == self.chat_messages.len() - 1 && self.is_waiting_response
                                    {
                                        // Show streaming response for the last assistant message
                                        if !self.current_response.is_empty() {
                                            ui.vertical(|ui| {
                                                ui.scope(|ui| {
                                                    ui.set_max_width(ui.available_width());
                                                    ui.style_mut().wrap_mode =
                                                        Some(egui::TextWrapMode::Wrap);
                                                    if search_query.is_empty() {
                                                        CommonMarkViewer::new()
                                                            .max_image_width(Some(
                                                                ui.available_width() as usize,
                                                            ))
                                                            .show(
                                                                ui,
                                                                &mut self.markdown_cache,
                                                                &self.current_response,
                                                            );
                                                    } else {
                                                        self.render_highlighted_text(
                                                            ui,
                                                            &self.current_response,
                                                            &search_query,
                                                        );
                                                    }
                                                });

                                                // Add buttons at the end of streaming message
                                                ui.horizontal(|ui| {
                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            if ui
                                                                .small_button(egui::RichText::new("🗑").color(egui::Color32::from_rgb(0x8E, 0x94, 0x9B)))
                                                                .on_hover_text("Delete message")
                                                                .clicked()
                                                            {
                                                                message_to_delete = Some(i);
                                                            }
                                                            if ui.small_button(egui::RichText::new("🗄 Memory").color(egui::Color32::from_rgb(0x8E, 0x94, 0x9B))).clicked()
                                                            {
                                                                memory_actions.push((
                                                                    self.current_response.clone(),
                                                                    "assistant".to_owned(),
                                                                ));
                                                            }
                                                            if ui.button(egui::RichText::new("📌 Digest").color(egui::Color32::from_rgb(0x8E, 0x94, 0x9B))).clicked() {
                                                                digest_actions.push((
                                                                    self.current_response.clone(),
                                                                    "assistant".to_owned(),
                                                                ));
                                                            }
                                                        },
                                                    );
                                                });
                                            });
                                        } else {
                                            ui.colored_label(egui::Color32::BROWN, "🖊 typing...");
                                        }
                                    } else {
                                        ui.vertical(|ui| {
                                            ui.scope(|ui| {
                                                ui.set_max_width(ui.available_width());
                                                ui.style_mut().wrap_mode =
                                                    Some(egui::TextWrapMode::Wrap);
                                                if search_query.is_empty() {
                                                    CommonMarkViewer::new().show(
                                                        ui,
                                                        &mut self.markdown_cache,
                                                        &message_content,
                                                    );
                                                } else {
                                                    self.render_highlighted_text(
                                                        ui,
                                                        &message_content,
                                                        &search_query,
                                                    );
                                                }
                                            });

                                            // Add buttons at the end of message
                                            ui.horizontal(|ui| {
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        if ui
                                                            .small_button(egui::RichText::new("🗑").color(egui::Color32::from_rgb(0x8E, 0x94, 0x9B)))
                                                            .on_hover_text("Delete message")
                                                            .clicked()
                                                        {
                                                            message_to_delete = Some(i);
                                                        }
                                                        if ui.small_button(egui::RichText::new("🗄 Memory").color(egui::Color32::from_rgb(0x8E, 0x94, 0x9B))).clicked()
                                                        {
                                                            memory_actions.push((
                                                                message_content.clone(),
                                                                message_role.clone(),
                                                            ));
                                                        }
                                                        if ui.button(egui::RichText::new("📌 Digest").color(egui::Color32::from_rgb(0x8E, 0x94, 0x9B))).clicked() {
                                                            digest_actions.push((
                                                                message_content.clone(),
                                                                message_role.clone(),
                                                            ));
                                                        }
                                                    },
                                                );
                                            });
                                        });
                                    }
                                    // Add spacing after assistant response (end of conversation turn)
                                    ui.add_space(8.0);
                                }
                            }
                        }
                    }

                    if let Some(error) = &self.last_error {
                        ui.colored_label(egui::Color32::RED, format!("Error: {error}"));
                    }
                });

                // Scroll to bottom when explicitly requested (Enter key or Send button)
                if self.should_scroll_chat {
                    scroll_output.state.offset.y = f32::INFINITY;
                    self.should_scroll_chat = false;
                }

                // Handle message deletion
                if let Some(index) = message_to_delete {
                    self.chat_messages.remove(index);
                }
            });

        (digest_actions, memory_actions)
    }
}
