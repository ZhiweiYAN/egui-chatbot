use crate::app::TemplateApp;
use egui_commonmark::CommonMarkViewer;

impl TemplateApp {
    pub fn render_long_mem_panel(&mut self, ctx: &egui::Context) {
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
                            self.should_scroll_chat = true;
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
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if self.long_term_memory_items.is_empty() {
                            ui.colored_label(egui::Color32::GRAY, "No memory items yet.\nClick 'M Memory' on chat messages to store important content.");
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
                                            if self.long_term_memory_items[i].source == "user" { egui::Color32::LIGHT_BLUE } else { egui::Color32::DARK_GREEN },
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
                                    if self.long_term_memory_items[i].source == "user" {
                                        ui.label(&self.long_term_memory_items[i].content);
                                    } else {
                                        // Render assistant messages as markdown in long term memory panel
                                        CommonMarkViewer::new().show(ui, &mut self.markdown_cache, &self.long_term_memory_items[i].content);
                                    }
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
    }
}