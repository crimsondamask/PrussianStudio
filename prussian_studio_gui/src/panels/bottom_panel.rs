use crate::app::TemplateApp;
use egui::{Context, Grid, InnerResponse, Window};

pub fn bottom_panel(ctx: &Context, _app: &mut TemplateApp) -> InnerResponse<()> {
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("powered by ");
            ui.hyperlink_to("PrussianStudio Nano", "https://github.com/crimsondamask");
            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                ui.spacing_mut().item_spacing.x = 200.0;
                ui.label("v0.1");
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 20.0;
                    ui.spinner();
                    ui.label("Status: Waiting...");
                });
            });
        });
    })
}
