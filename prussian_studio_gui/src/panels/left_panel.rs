use crate::app::TemplateApp;
use egui::{Context, InnerResponse};
use std::process::Command;

pub fn left_panel(ctx: &Context, app: &mut TemplateApp) -> InnerResponse<()> {
    // egui::SidePanel::left("side_panel_left").show(ctx, |ui| {
    //     ui.label("Information");
    //     ui.separator();
    //     ui.label(format!("{}", app.value));
    // })

    egui::SidePanel::left("side_panel").show(ctx, |ui| {
        ui.label("Options");
        ui.separator();
        ui.group(|ui| {
            if ui.button("HMI").clicked() {
                Command::new("./PrussianStudio_HMI")
                    .spawn()
                    .expect("command failed to start");
            }
            ui.toggle_value(&mut app.spawn_logging_thread, "Logging");
        });
        // for channel in app.devices[0].channels.clone() {
        // ui.label(format!("{}", channel.id));
        // }
    })
}
