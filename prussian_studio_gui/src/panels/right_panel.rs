use crate::app::TemplateApp;
use egui::{Context, Grid, InnerResponse};

const NUM_COLUMNS: usize = 2;

pub fn right_panel(ctx: &Context, app: &TemplateApp) -> InnerResponse<()> {
    egui::SidePanel::right("side_panel_right").show(ctx, |ui| {
        ui.label("Devices Information");
        ui.separator();
        Grid::new("right_panel").num_columns(2).show(ui, |ui| {
            ui.label("Name");
            ui.label("Status");
            ui.end_row();
            for _ in 0..NUM_COLUMNS {
                ui.separator();
            }
            ui.end_row();
            ui.label(format!("{}", app.devices[0]));
            ui.label(format!("{}", app.devices[0].status));
            ui.end_row();
            ui.label(format!("{}", app.devices[1]));
            ui.label(format!("{}", app.devices[1].status));
            ui.end_row();
        });
    })
}
