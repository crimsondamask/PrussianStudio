use crate::app::TemplateApp;
use egui::{Context, InnerResponse};

pub fn central_panel(ctx: &Context, _app: &mut TemplateApp) -> InnerResponse<()> {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.label("Monitor");
        ui.separator();

        // We check if the logging button has been pressed to spawn our threads.

        egui::warn_if_debug_build(ui);

        // We check if there is any new message from the HMI recv thread and update the corresponding device.
        // write_channel_from_hmi(app);
    })
}
