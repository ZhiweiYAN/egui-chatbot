use crate::app::TemplateApp;
use egui_commonmark::CommonMarkViewer;

impl TemplateApp {
    #[expect(clippy::too_many_lines)]
    pub fn render_long_mem_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("long_term_memory")
            .default_width(400.0)
            .min_width(300.0)
            .max_width(500.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("🗄 Longterm Memory");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let selected_count = self.long_term_memory_items.iter().filter(|item| item.selected).count();
                        let summary_enabled = selected_count > 0 && !self.is_waiting_response;
                        let button_text = if self.is_waiting_response {
                            "🤖 Processing...".to_owned()
                        } else if selected_count > 0 {
                            format!("📄 Summary ({selected_count})")
                        } else {
                            "📄 Summary".to_owned()
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
                            let search_query = self.memory_search.clone(); // Keep original case for highlighting
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
                                    // Check selection state first
                                    let is_selected = self.long_term_memory_items[i].selected;

                                    // Apply background highlighting if selected
                                    if is_selected {
                                        egui::Frame::default()
                                            .fill(ui.visuals().selection.bg_fill)
                                            .inner_margin(egui::Margin::same(4))
                                            .corner_radius(egui::CornerRadius::same(3))
                                            .show(ui, |ui| {
                                                // Header with checkbox, label, and timestamp
                                                ui.horizontal(|ui| {
                                                    ui.checkbox(&mut self.long_term_memory_items[i].selected, "");
                                                    let source_label = if self.long_term_memory_items[i].source == "user" { "You" } else { "Assistant" };
                                                    ui.colored_label(
                                                        if self.long_term_memory_items[i].source == "user" { egui::Color32::DARK_RED } else { egui::Color32::DARK_GREEN },
                                                        format!("{source_label}:")
                                                    );
                                                    ui.label(&self.long_term_memory_items[i].timestamp);
                                                });

                                                // Content
                                                if self.long_term_memory_items[i].source == "user" {
                                                    self.render_highlighted_text(ui, &self.long_term_memory_items[i].content, &search_query);
                                                } else {
                                                    // For assistant messages, use highlighting if there's a search term, otherwise use markdown
                                                    if search_query.is_empty() {
                                                        // Render assistant messages as markdown in long term memory panel
                                                        CommonMarkViewer::new().show(ui, &mut self.markdown_cache, &self.long_term_memory_items[i].content);
                                                    } else {
                                                        // Render with highlighting (plain text)
                                                        self.render_highlighted_text(ui, &self.long_term_memory_items[i].content, &search_query);
                                                    }
                                                }

                                                // Action buttons at the end
                                                ui.horizontal(|ui| {
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if ui.small_button("🗑").on_hover_text("Delete item").clicked() {
                                                            item_to_delete = Some(i);
                                                        }
                                                        if ui.small_button("📋").on_hover_text("Copy to clipboard").clicked() {
                                                            ui.ctx().copy_text(self.long_term_memory_items[i].content.clone());
                                                        }
                                                    });
                                                });
                                            });
                                    } else {
                                        // Header with checkbox, label, and timestamp
                                        ui.horizontal(|ui| {
                                            ui.checkbox(&mut self.long_term_memory_items[i].selected, "");
                                            let source_label = if self.long_term_memory_items[i].source == "user" { "You" } else { "Assistant" };
                                            ui.colored_label(
                                                if self.long_term_memory_items[i].source == "user" { egui::Color32::DARK_RED } else { egui::Color32::DARK_GREEN },
                                                format!("{source_label}:")
                                            );
                                            ui.label(&self.long_term_memory_items[i].timestamp);
                                        });

                                        // Content
                                        if self.long_term_memory_items[i].source == "user" {
                                            self.render_highlighted_text(ui, &self.long_term_memory_items[i].content, &search_query);
                                        } else {
                                            // For assistant messages, use highlighting if there's a search term, otherwise use markdown
                                            if search_query.is_empty() {
                                                // Render assistant messages as markdown in long term memory panel
                                                CommonMarkViewer::new().show(ui, &mut self.markdown_cache, &self.long_term_memory_items[i].content);
                                            } else {
                                                // Render with highlighting (plain text)
                                                self.render_highlighted_text(ui, &self.long_term_memory_items[i].content, &search_query);
                                            }
                                        }

                                        // Action buttons at the end
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("🗑").on_hover_text("Delete item").clicked() {
                                                    item_to_delete = Some(i);
                                                }
                                                if ui.small_button("📋").on_hover_text("Copy to clipboard").clicked() {
                                                    ui.ctx().copy_text(self.long_term_memory_items[i].content.clone());
                                                }
                                            });
                                        });
                                    }

                                    ui.add_space(5.0);
                                }
                            }
                        }
                    });

                if let Some(index) = item_to_delete {
                    self.long_term_memory_items.remove(index);
                }

                ui.add_space(-30.0);
                ui.separator();

                ui.horizontal(|ui| {
                    // Always show buttons, but enable/disable based on content
                    let clear_enabled = !self.long_term_memory_items.is_empty();
                    let export_enabled = !self.long_term_memory_items.is_empty();

                    if ui.add_enabled(clear_enabled, egui::Button::new("Clear All")).clicked() {
                        // Clear longterm memory panel associations from database (soft delete)
                        if let Some(ref db) = self.database {
                            if let Err(e) = db.clear_longterm_panel_associations() {
                                log::error!("Failed to clear longterm panel associations: {e}");
                                self.last_error = Some(format!("Database error: {e}"));
                            }
                        }

                        // Clear UI state
                        self.long_term_memory_items.clear();
                    }
                    if ui.add_enabled(export_enabled, egui::Button::new("Export All")).clicked() {
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
