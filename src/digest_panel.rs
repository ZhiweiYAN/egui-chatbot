use crate::app::TemplateApp;
use egui_commonmark::CommonMarkViewer;

impl TemplateApp {
    pub fn render_digest_panel(&mut self, ctx: &egui::Context) -> Vec<(String, String)> {
        let mut memory_actions = Vec::new();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(-6.0);
            ui.horizontal(|ui| {
                ui.heading("📌 Digested Content");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let selected_count = self.digest_items.iter().filter(|item| item.selected).count();
                    let summary_enabled = selected_count > 0 && !self.is_waiting_response;
                    let button_text = if self.is_waiting_response {
                        "LLM Processing...".to_string()
                    } else if selected_count > 0 {
                        format!("📄 Summary ({})", selected_count)
                    } else {
                        "📄 Summary".to_string()
                    };

                    if ui.add_enabled(summary_enabled, egui::Button::new(button_text))
                        .on_hover_text("Generate a summary of selected digest items and show in chat")
                        .clicked()
                    {
                        self.start_digest_summary_generation(ui.ctx());
                        self.should_scroll_chat = true;
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

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if self.digest_items.is_empty() {
                        ui.colored_label(egui::Color32::GRAY, "No digest items yet.\nClick '📌 Digest' on chat messages to collect important content.");
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
                                        if self.digest_items[i].source == "user" { egui::Color32::LIGHT_BLUE } else { egui::Color32::DARK_GREEN },
                                        &format!("{}:", if self.digest_items[i].source == "user" { "You" } else { "Assistant" })
                                    );
                                    ui.label(&self.digest_items[i].timestamp);
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("🗑").on_hover_text("Delete item").clicked() {
                                            item_to_delete = Some(i);
                                        }
                                        if ui.small_button("🗄").on_hover_text("Copy to Long Term Memory").clicked() {
                                            memory_actions.push((self.digest_items[i].content.clone(), self.digest_items[i].source.clone()));
                                        }
                                        if ui.small_button("📋").on_hover_text("Copy to clipboard").clicked() {
                                            ui.ctx().copy_text(self.digest_items[i].content.clone());
                                        }
                                    });
                                });
                                if self.digest_items[i].source == "user" {
                                    ui.label(&self.digest_items[i].content);
                                } else {
                                    // Render assistant messages as markdown in digest panel
                                    CommonMarkViewer::new().show(ui, &mut self.markdown_cache, &self.digest_items[i].content);
                                }
                                ui.add_space(5.0);
                            }
                        }
                    }
                });

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

        memory_actions
    }
}
