use crate::app::TemplateApp;
use egui::{Context, InnerResponse};
use std::process::Command;

pub fn left_panel(ctx: &Context, app: &mut TemplateApp) -> InnerResponse<()> {
    egui::SidePanel::left("side_panel").show(ctx, |ui| {
        ui.label("Options");
        ui.separator();
        ui.group(|ui| {
            // ui.toggle_value(&mut app.spawn_logging_thread, "Logging");
            if ui.button("Launch HMI").clicked() {
                Command::new("./PrussianStudio_HMI")
                    .spawn()
                    .expect("command failed to start");
            }
        });
    })
}
