use egui::Color32;

/// Test color scheme from the provided palette
pub struct ColorScheme {
    pub background: Color32,
    pub surface: Color32,
    pub card_background: Color32,
    pub primary_text: Color32,
    pub secondary_text: Color32,
    pub muted_text: Color32,
    pub accent_blue: Color32,
    pub icon: Color32,
    pub border: Color32,
    pub nav_background: Color32,
}

impl ColorScheme {
    pub fn light_mode() -> Self {
        Self {
            background: Color32::from_rgb(0xE8, 0xEC, 0xF1),      // #E8ECF1
            surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),         // #FFFFFF
            card_background: Color32::from_rgb(0xEF, 0xE8, 0xE6), // #EFE8E6
            primary_text: Color32::from_rgb(0x0E, 0x15, 0x21),    // #0E1521
            secondary_text: Color32::from_rgb(0x8E, 0x94, 0x9B),  // #8E949B
            muted_text: Color32::from_rgb(0xA7, 0xAB, 0xB0),      // #A7ABB0
            accent_blue: Color32::from_rgb(0xC2, 0xDE, 0xFF),     // #C2DEFF
            icon: Color32::from_rgb(0x0E, 0x15, 0x21),            // #0E1521
            border: Color32::from_rgb(0xE6, 0xEA, 0xED),          // #E6EAED
            nav_background: Color32::from_rgb(0xFB, 0xFB, 0xFC),  // #FBFBFC
        }
    }
}

/// Demo window to test the color scheme
#[expect(clippy::too_many_lines)]
pub fn show_color_test_window(ctx: &egui::Context, open: &mut bool) {
    let colors = ColorScheme::light_mode();

    egui::Window::new("🎨 Color Scheme Test")
        .open(open)
        .resizable(true)
        .default_width(600.0)
        .default_height(500.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.heading("Light Mode Color Palette Test");
                    ui.separator();

            // Color swatches
            ui.label("Color Swatches:");
            ui.horizontal_wrapped(|ui| {
                show_color_swatch(ui, "Background", colors.background, "#E8ECF1");
                show_color_swatch(ui, "Surface", colors.surface, "#FFFFFF");
                show_color_swatch(ui, "Card BG", colors.card_background, "#EFE8E6");
            });

            ui.horizontal_wrapped(|ui| {
                show_color_swatch(ui, "Primary Text", colors.primary_text, "#0E1521");
                show_color_swatch(ui, "Secondary Text", colors.secondary_text, "#8E949B");
                show_color_swatch(ui, "Muted Text", colors.muted_text, "#A7ABB0");
            });

            ui.horizontal_wrapped(|ui| {
                show_color_swatch(ui, "Accent Blue", colors.accent_blue, "#C2DEFF");
                show_color_swatch(ui, "Icon", colors.icon, "#0E1521");
                show_color_swatch(ui, "Border", colors.border, "#E6EAED");
                show_color_swatch(ui, "Nav BG", colors.nav_background, "#FBFBFC");
            });

            ui.separator();
            ui.heading("UI Component Preview");

            // Background simulation
            let bg_frame = egui::Frame::new()
                .fill(colors.background)
                .inner_margin(12.0)
                .corner_radius(8.0);

            bg_frame.show(ui, |ui| {
                ui.label(egui::RichText::new("Application Background Area")
                    .color(colors.primary_text));

                ui.add_space(8.0);

                // Surface card
                let surface_frame = egui::Frame::new()
                    .fill(colors.surface)
                    .stroke(egui::Stroke::new(1.0, colors.border))
                    .inner_margin(10.0)
                    .corner_radius(6.0);

                surface_frame.show(ui, |ui| {
                    ui.label(egui::RichText::new("Surface Card")
                        .color(colors.primary_text)
                        .strong());
                    ui.label(egui::RichText::new("This is primary text on a surface")
                        .color(colors.primary_text));
                    ui.label(egui::RichText::new("This is secondary text with less emphasis")
                        .color(colors.secondary_text));
                    ui.label(egui::RichText::new("This is muted text for hints")
                        .color(colors.muted_text));
                });

                ui.add_space(8.0);

                // Card background
                let card_frame = egui::Frame::new()
                    .fill(colors.card_background)
                    .stroke(egui::Stroke::new(1.0, colors.border))
                    .inner_margin(10.0)
                    .corner_radius(6.0);

                card_frame.show(ui, |ui| {
                    ui.label(egui::RichText::new("Card Background")
                        .color(colors.primary_text)
                        .strong());
                    ui.label(egui::RichText::new("User message example")
                        .color(colors.primary_text));
                    ui.horizontal(|ui| {
                        _ = ui.button("📌 Digest");
                        _ = ui.button("🗄 Memory");
                    });
                });

                ui.add_space(8.0);

                // Accent area
                let accent_frame = egui::Frame::new()
                    .fill(colors.accent_blue)
                    .stroke(egui::Stroke::new(1.0, colors.border))
                    .inner_margin(10.0)
                    .corner_radius(6.0);

                accent_frame.show(ui, |ui| {
                    ui.label(egui::RichText::new("Accent Blue Area")
                        .color(colors.primary_text)
                        .strong());
                    ui.label(egui::RichText::new("Highlighted or selected content")
                        .color(colors.primary_text));
                });

                ui.add_space(8.0);

                // Navigation bar simulation
                let nav_frame = egui::Frame::new()
                    .fill(colors.nav_background)
                    .stroke(egui::Stroke::new(1.0, colors.border))
                    .inner_margin(10.0)
                    .corner_radius(6.0);

                nav_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("📁")
                            .color(colors.icon)
                            .size(16.0));
                        ui.label(egui::RichText::new("Navigation Item")
                            .color(colors.primary_text));

                        ui.add_space(20.0);

                        ui.label(egui::RichText::new("⚙")
                            .color(colors.icon)
                            .size(16.0));
                        ui.label(egui::RichText::new("Settings")
                            .color(colors.secondary_text));
                    });
                });
            });

            ui.separator();
            ui.heading("Chat Panel Simulation");

            // Chat panel background
            let chat_bg_frame = egui::Frame::new()
                .fill(colors.nav_background)
                .stroke(egui::Stroke::new(1.0, colors.border))
                .inner_margin(12.0)
                .corner_radius(8.0);

            chat_bg_frame.show(ui, |ui| {
                ui.label(egui::RichText::new("💬 Chat History")
                    .color(colors.primary_text)
                    .strong());

                ui.add_space(6.0);

                // User message
                ui.label(egui::RichText::new("You:")
                    .color(colors.primary_text)
                    .strong());

                let user_msg_frame = egui::Frame::new()
                    .fill(colors.card_background)
                    .inner_margin(8.0)
                    .corner_radius(4.0);

                user_msg_frame.show(ui, |ui| {
                    ui.label(egui::RichText::new("Hello! How are you?")
                        .color(colors.primary_text));
                });

                ui.add_space(6.0);

                // Assistant message
                ui.label(egui::RichText::new("Assistant:")
                    .color(colors.secondary_text)
                    .strong());

                let assistant_msg_frame = egui::Frame::new()
                    .fill(colors.surface)
                    .stroke(egui::Stroke::new(1.0, colors.border))
                    .inner_margin(8.0)
                    .corner_radius(4.0);

                assistant_msg_frame.show(ui, |ui| {
                    ui.label(egui::RichText::new("I'm doing well, thank you! How can I help you today?")
                        .color(colors.primary_text));
                });
            });

            ui.separator();
            ui.label(egui::RichText::new("Tip: Compare colors side-by-side to check contrast and readability")
                .color(colors.muted_text)
                .italics());
                });
        });
}

fn show_color_swatch(ui: &mut egui::Ui, name: &str, color: Color32, hex: &str) {
    ui.vertical(|ui| {
        ui.set_min_width(100.0);

        // Color box using frame
        let frame = egui::Frame::new()
            .fill(color)
            .stroke(egui::Stroke::new(1.0, Color32::GRAY))
            .corner_radius(4.0);

        frame.show(ui, |ui| {
            ui.allocate_space(egui::vec2(80.0, 40.0));
        });

        // Labels
        ui.small(name);
        ui.small(hex);
    });
}
