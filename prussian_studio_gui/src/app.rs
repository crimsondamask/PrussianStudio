// use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::{
    fonts::*,
    panels::{central_panel::central_panel, left_panel::left_panel, right_panel::right_panel, *},
    window::*,
};
use egui::{global_dark_light_mode_buttons, mutex::Mutex, Color32, ComboBox, Rounding, Window};
use egui::{Button, Grid, Slider};
pub use lib_device::Channel;
pub use lib_device::*;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    pub device_windows_buffer: DeviceWindowsBuffer,
    #[serde(skip)]
    pub channel_windows_buffer: ChannelWindowsBuffer,
    #[serde(skip)]
    pub value: f32,
    #[serde(skip)]
    pub windows_open: WindowsOpen,
    #[serde(skip)]
    pub devices: Vec<Device>,
    #[serde(skip)]
    pub mpsc_channel: Option<(Sender<Vec<Device>>, Receiver<Vec<Device>>)>,
    #[serde(skip)]
    pub spawn_logging_thread: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            device_windows_buffer: DeviceWindowsBuffer::default(),
            channel_windows_buffer: ChannelWindowsBuffer::default(),
            value: 2.7,
            windows_open: WindowsOpen::default(),
            devices: vec![
                Device {
                    name: "PLC".to_owned(),
                    ..Default::default()
                },
                Device {
                    name: "Modbus Device".to_owned(),
                    ..Default::default()
                },
            ],
            mpsc_channel: None,
            spawn_logging_thread: false,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        setup_custom_fonts(&cc.egui_ctx);

        let visuals = egui::Visuals {
            dark_mode: false,
            // override_text_color: Some(Color32::GRAY),
            window_rounding: Rounding {
                nw: 7.0,
                ne: 7.0,
                sw: 7.0,
                se: 7.0,
            },
            hyperlink_color: Color32::from_rgb(0, 142, 240),
            // faint_bg_color: Color32::from_gray(200),
            // override_text_color: Some(Color32::from_gray(200)),
            // ..Default::default()
            ..egui::Visuals::light()
        };
        cc.egui_ctx.set_visuals(visuals);
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            device_windows_buffer,
            channel_windows_buffer,
            value,
            windows_open,
            devices,
            mpsc_channel,
            spawn_logging_thread,
        } = self;

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("powered by ");
                ui.hyperlink_to("PrussianStudio", "https://github.com/crimsondamask");
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
        });
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Preferences").clicked() {
                        windows_open.preferences = !windows_open.preferences;
                    }
                });
                ui.menu_button("Devices", |ui| {
                    if ui.button("Add new device").clicked() {
                        windows_open.new_device = !windows_open.new_device;
                    }
                    ui.separator();
                    ui.menu_button("PLC", |ui| {
                        if ui.button("Configure").clicked() {
                            windows_open.plc = !windows_open.plc;
                        }
                        if ui.button("Channels").clicked() {
                            windows_open.device_channels = !windows_open.device_channels;
                        }
                    });
                    if ui.button("Modbus Device").clicked() {
                        windows_open.modbus_device = !windows_open.modbus_device;
                    }
                });
                ui.menu_button("Help", |ui| if ui.button("About").clicked() {});
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    // global_dark_light_mode_buttons(ui);
                });
            });
            Window::new("PLC Channels")
                .open(&mut windows_open.device_channels)
                .scroll2([false, true])
                .show(ctx, |ui| {
                    Grid::new("Channel List")
                        .striped(true)
                        .num_columns(6)
                        .min_col_width(160.0)
                        .show(ui, |ui| {
                            if let Some(device) = &devices.iter().nth(0) {
                                ui.label("Device status:");
                                ui.label(format!("{}", &device.status));

                                ui.end_row();
                                ui.separator();
                                ui.end_row();
                                ui.label("Channel");
                                ui.label("Value");
                                ui.label("Value type");
                                ui.label("Access");
                                ui.label("Address");
                                ui.label("Status");
                                ui.end_row();
                                for _ in 0..6 {
                                    ui.separator();
                                }
                                ui.end_row();
                                for channel in &device.channels {
                                    let button =
                                        Button::new(format!("CH{}", channel.id)).frame(true);
                                    if ui.add(button).clicked() {
                                        channel_windows_buffer.selected_channel = channel.clone();
                                        windows_open.channel_config = !windows_open.channel_config;
                                    }
                                    // ui.label(format!("CH{}", channel.id));
                                    ui.label(format!("{:.2}", channel.value));
                                    ui.label(format!("{}", channel.value_type));
                                    ui.label(format!("{}", channel.access_type));
                                    ui.label(format!("{}", channel.index));
                                    ui.label(format!("{}", channel.status));
                                    // if ui.small_button("Configure").clicked() {}
                                    ui.end_row();
                                }
                            }
                        });
                });
            Window::new("Preferences")
                .open(&mut windows_open.preferences)
                .show(ctx, |ui| {
                    ctx.settings_ui(ui);
                });
            Window::new("Channel Configuration")
                .open(&mut windows_open.channel_config)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "{} configuration",
                        channel_windows_buffer.selected_channel
                    ));
                    ui.separator();
                    egui::Grid::new("Channel config")
                        .num_columns(2)
                        .show(ui, |ui| {
                            // ui.text_edit_singleline(&mut channel_windows_buffer.index);
                            ui.add(
                                Slider::new(
                                    &mut channel_windows_buffer.edited_channel.index,
                                    0..=49999,
                                )
                                .text("Index"),
                            );
                            ui.end_row();
                            ComboBox::from_label("Value type")
                                .selected_text(format!(
                                    "{}",
                                    channel_windows_buffer.edited_channel.value_type
                                ))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut channel_windows_buffer.edited_channel.value_type,
                                        ValueType::Int16,
                                        format!("{}", ValueType::Int16),
                                    );
                                    ui.selectable_value(
                                        &mut channel_windows_buffer.edited_channel.value_type,
                                        ValueType::Real32,
                                        format!("{}", ValueType::Real32),
                                    );
                                    ui.selectable_value(
                                        &mut channel_windows_buffer.edited_channel.value_type,
                                        ValueType::BoolType,
                                        format!("{}", ValueType::BoolType),
                                    );
                                });
                            ui.end_row();
                            ComboBox::from_label("Access type")
                                .selected_text(format!(
                                    "{}",
                                    channel_windows_buffer.edited_channel.access_type
                                ))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut channel_windows_buffer.edited_channel.access_type,
                                        AccessType::Read,
                                        format!("{}", AccessType::Read),
                                    );
                                    ui.selectable_value(
                                        &mut channel_windows_buffer.edited_channel.access_type,
                                        AccessType::Write,
                                        format!("{}", AccessType::Write),
                                    );
                                });
                            ui.end_row();
                        });
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Save").clicked() {
                            let device = &mut devices[0];
                            device.channels[channel_windows_buffer.selected_channel.id] =
                                channel_windows_buffer.edited_channel.clone();
                        }
                    });
                });
            Window::new("PLC")
                .open(&mut windows_open.plc)
                .show(ctx, |ui| {
                    ui.label("Configuration");
                    ui.separator();
                    egui::Grid::new("add_device").num_columns(2).show(ui, |ui| {
                        ui.label("Device name:");
                        ui.text_edit_singleline(&mut device_windows_buffer.name);
                        ui.end_row();
                        ui.label("IP address:");
                        ui.text_edit_singleline(&mut device_windows_buffer.address);
                        ui.end_row();
                        ui.label("Port:");
                        ui.text_edit_singleline(&mut device_windows_buffer.port);
                        ui.end_row();
                    });
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Save").clicked() {
                            if let Ok(port) = device_windows_buffer.port.parse::<usize>() {
                                devices[0].name = device_windows_buffer.name.clone();
                                devices[0].config.address = device_windows_buffer.address.clone();
                                devices[0].config.port = port;
                            }
                        }
                    });
                });
            Window::new("Modbus Device")
                .open(&mut windows_open.modbus_device)
                .scroll2([false, true])
                .show(ctx, |ui| {});
            Window::new("Add Device")
                .open(&mut windows_open.new_device)
                .show(ctx, |ui| {});
        });

        right_panel(ctx, &self);
        left_panel(ctx, self);
        central_panel(ctx, self);
    }
}
