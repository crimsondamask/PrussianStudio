use crate::app::TemplateApp;
use egui::{Context, Grid, InnerResponse};

pub fn right_panel(ctx: &Context, app: &TemplateApp) -> InnerResponse<()> {
    egui::SidePanel::right("side_panel_right").show(ctx, |ui| {
        ui.label("Information");
        ui.separator();
        Grid::new("right_panel").show(ui, |ui| {
            ui.label("Devices:");
            ui.label(format!("{}", app.devices.len()));
        });
    })
}
