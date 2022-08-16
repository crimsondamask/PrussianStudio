use crate::app::TemplateApp;
use egui::{Context, Grid, InnerResponse, Window};

pub fn top_panel(ctx: &Context, app: &mut TemplateApp) -> InnerResponse<()> {
    egui::TopBottomPanel::top("Top panel").show(ctx, |ui| {
        // The top panel is often a good place for a menu bar:
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    // frame.quit();
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.button("Preferences").clicked() {
                    app.windows_open.preferences = !app.windows_open.preferences;
                }
            });
            ui.menu_button("Devices", |ui| {
                if ui.button("Add new device").clicked() {
                    app.windows_open.new_device = !app.windows_open.new_device;
                }
                ui.separator();
                ui.menu_button("PLC", |ui| {
                    if ui.button("Configure").clicked() {
                        app.windows_open.plc = !app.windows_open.plc;
                    }
                    if ui.button("Channels").clicked() {
                        app.windows_open.device_channels = !app.windows_open.device_channels;
                    }
                });
                if ui.button("Modbus Device").clicked() {
                    app.windows_open.modbus_device = !app.windows_open.modbus_device;
                }
            });
            ui.menu_button("Help", |ui| if ui.button("About").clicked() {});
            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                // global_dark_light_mode_buttons(ui);
            });
        });
        // Window::new("PLC Channels")
        //     .open(&mut app.windows_open.device_channels)
        //     .scroll2([false, true])
        //     .show(ctx, |ui| {
        //         Grid::new("Channel List")
        //             .striped(true)
        //             .num_columns(6)
        //             .min_col_width(160.0)
        //             .show(ui, |ui| {
        //                 if let Some(device) = &app.devices.iter().nth(0) {
        //                     ui.label("Channel");
        //                     ui.label("Value");
        //                     ui.label("Value type");
        //                     ui.label("Access");
        //                     ui.label("Address");
        //                     ui.end_row();
        //                     for _ in 0..6 {
        //                         ui.separator();
        //                     }
        //                     ui.end_row();
        //                     for channel in &device.channels {
        //                         ui.label(format!("CH{}", channel.id));
        //                         ui.label(format!("{:.1}", channel.value));
        //                         ui.label(format!("{}", channel.value_type));
        //                         ui.label(format!("{}", channel.access_type));
        //                         ui.label(format!("{}", channel.index));
        //                         if ui.small_button("Configure").clicked() {
        //                             app.channel_windows_buffer.selected_channel = channel.clone();
        //                             app.windows_open.channel_config =
        //                                 !app.windows_open.channel_config;
        //                         }
        //                         ui.end_row();
        //                     }
        //                 }
        //             });
        //     });
    })
}
