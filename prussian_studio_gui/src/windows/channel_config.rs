use crate::app::TemplateApp;
use egui::{ComboBox, Context, InnerResponse, Slider, Window};
use lib_device::*;

pub fn channel_config_window(
    ctx: &Context,
    app: &mut TemplateApp,
) -> Option<InnerResponse<Option<()>>> {
    Window::new("Channel Configuration")
        .open(&mut app.windows_open.channel_config)
        .show(ctx, |ui| {
            ui.label(format!(
                "{} configuration",
                app.channel_windows_buffer.selected_channel
            ));
            ui.separator();
            egui::Grid::new("Channel config")
                .num_columns(2)
                .show(ui, |ui| {
                    // ui.text_edit_singleline(&mut channel_windows_buffer.index);
                    ui.add(
                        Slider::new(
                            &mut app.channel_windows_buffer.edited_channel.index,
                            0..=49999,
                        )
                        .text("Index"),
                    );
                    ui.end_row();
                    ComboBox::from_label("Value type")
                        .selected_text(format!(
                            "{}",
                            app.channel_windows_buffer.edited_channel.value_type
                        ))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.channel_windows_buffer.edited_channel.value_type,
                                ValueType::Int16,
                                format!("{}", ValueType::Int16),
                            );
                            ui.selectable_value(
                                &mut app.channel_windows_buffer.edited_channel.value_type,
                                ValueType::Real32,
                                format!("{}", ValueType::Real32),
                            );
                            ui.selectable_value(
                                &mut app.channel_windows_buffer.edited_channel.value_type,
                                ValueType::BoolType,
                                format!("{}", ValueType::BoolType),
                            );
                        });
                    ui.end_row();
                    ComboBox::from_label("Access type")
                        .selected_text(format!(
                            "{}",
                            app.channel_windows_buffer.edited_channel.access_type
                        ))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.channel_windows_buffer.edited_channel.access_type,
                                AccessType::Read,
                                format!("{}", AccessType::Read),
                            );
                            ui.selectable_value(
                                &mut app.channel_windows_buffer.edited_channel.access_type,
                                AccessType::Write,
                                format!("{}", AccessType::Write),
                            );
                        });
                    ui.end_row();
                });
            ui.vertical_centered_justified(|ui| {
                if ui.button("Save").clicked() {
                    let device = &mut app.devices[0];
                    device.channels[app.channel_windows_buffer.selected_channel.id] =
                        app.channel_windows_buffer.edited_channel;
                }
            });
        })
}
