use std::collections::BTreeMap;

use crate::{
    fonts::*,
    panels::{central_panel::central_panel, left_panel::left_panel, right_panel::right_panel, *},
    window::*,
};
use egui::{global_dark_light_mode_buttons, Color32, Rounding, Window};
use lib_device::*;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    pub device_windows_buffer: DeviceWindowsBuffer,
    #[serde(skip)]
    pub value: f32,
    #[serde(skip)]
    pub windows_open: WindowsOpen,
    pub devices: BTreeMap<String, Device>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            device_windows_buffer: DeviceWindowsBuffer::default(),
            value: 2.7,
            windows_open: WindowsOpen::default(),
            devices: BTreeMap::new(),
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
            dark_mode: true,
            // override_text_color: Some(Color32::GRAY),
            window_rounding: Rounding {
                nw: 7.0,
                ne: 7.0,
                sw: 7.0,
                se: 7.0,
            },
            hyperlink_color: Color32::from_rgb(0, 142, 240),
            // faint_bg_color: Color32::from_gray(200),
            override_text_color: Some(Color32::from_gray(200)),
            // button_frame: false,
            ..Default::default() // ..egui::Visuals::light()
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
            value,
            windows_open,
            devices,
        } = self;

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("powered by ");
                ui.hyperlink_to("PrussianStudio", "https://github.com/crimsondamask");
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.label("v0.1");
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
                ui.menu_button("Edit", |ui| if ui.button("Preferences").clicked() {});
                ui.menu_button("Devices", |ui| {
                    if ui.button("Add new device").clicked() {
                        windows_open.new_device = !windows_open.new_device;
                    }
                    ui.separator();
                    if ui.button("Modbus").clicked() {
                        windows_open.device = !windows_open.device;
                    }
                });
                ui.menu_button("Help", |ui| if ui.button("About").clicked() {});
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    // global_dark_light_mode_buttons(ui);
                });
            });
            Window::new("Modbus Devices")
                .open(&mut windows_open.device)
                .show(ctx, |ui| {
                    egui::Grid::new("devices").show(ui, |ui| {
                        let mut device_to_remove = String::new();
                        for (device, value) in devices.iter() {
                            ui.label(format!("{}    ", &device));
                            ui.label(format!("{}", &value.config.address));
                            if ui.small_button("Edit").clicked() {}
                            if ui.small_button("Remove").clicked() {
                                device_to_remove = device.to_owned();
                            }
                            ui.end_row();
                        }
                        if let Some(_) = devices.remove(&device_to_remove) {}
                    });
                });
            Window::new("Add Device")
                .open(&mut windows_open.new_device)
                .show(ctx, |ui| {
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
                        if ui.button("Add").clicked() {
                            if let Ok(port) = device_windows_buffer.port.parse::<usize>() {
                                let device_name = &device_windows_buffer.name;
                                let address = &device_windows_buffer.address;
                                let port = port;
                                let config = DeviceConfig {
                                    address: address.to_owned(),
                                    port,
                                };
                                let channels = Vec::new();
                                let device = Device::new(
                                    device_name.to_owned(),
                                    DeviceType::Modbus,
                                    config,
                                    channels,
                                );

                                devices.insert(device_name.to_owned(), device);
                            };
                        }
                    });
                });
        });

        right_panel(ctx, &self);
        left_panel(ctx, self);
        central_panel(ctx, self);
    }
}
