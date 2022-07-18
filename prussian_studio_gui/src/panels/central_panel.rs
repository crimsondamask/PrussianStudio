use crate::app::TemplateApp;
use egui::{Context, InnerResponse};

pub fn central_panel(ctx: &Context, _app: &mut TemplateApp) -> InnerResponse<()> {
    egui::CentralPanel::default().show(ctx, |ui| {
        // The central panel the region left after adding TopPanel's and SidePanel's

        ui.label("Monitor");
        ui.separator();
        egui::warn_if_debug_build(ui);
    })
}
