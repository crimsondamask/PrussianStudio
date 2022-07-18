use egui;

pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "custom_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../custom_font.otf")),
    );
    fonts.font_data.insert(
        "custom_font_mono".to_owned(),
        egui::FontData::from_static(include_bytes!("../custom_font_mono.ttf")),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "custom_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("custom_font_mono".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}
