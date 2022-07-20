use crate::app::TemplateApp;
use egui::{Context, InnerResponse};

pub fn left_panel(ctx: &Context, app: &mut TemplateApp) -> InnerResponse<()> {
    // egui::SidePanel::left("side_panel_left").show(ctx, |ui| {
    //     ui.label("Information");
    //     ui.separator();
    //     ui.label(format!("{}", app.value));
    // })

    egui::SidePanel::left("side_panel").show(ctx, |ui| {
        ui.label("Options");
        ui.separator();
        if ui.button("Fetch").clicked() {
            if let Ok(_) = app.devices[0].fetch_data_tcp() {}
        }
        // for channel in app.devices[0].channels.clone() {
        // ui.label(format!("{}", channel.id));
        // }
    })
}
