use crate::app::TemplateApp;
use egui::{Context, InnerResponse};

pub fn left_panel(ctx: &Context, _app: &mut TemplateApp) -> InnerResponse<()> {
    // egui::SidePanel::left("side_panel_left").show(ctx, |ui| {
    //     ui.label("Information");
    //     ui.separator();
    //     ui.label(format!("{}", app.value));
    // })

    egui::SidePanel::left("side_panel").show(ctx, |ui| {
        ui.label("Options");
        ui.separator();
    })
}
