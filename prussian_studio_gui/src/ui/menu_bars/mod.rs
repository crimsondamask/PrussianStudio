use ron::ser::{to_string_pretty, PrettyConfig};
use std::{fs, io::Write, path::PathBuf};

use extras::RetainedImage;
use lib_device::Device;
use lib_logger::Logger;

use crate::{
    config::AppConfig,
    status::Status,
    window::{ChannelWindowsBuffer, DeviceWindowsBuffer, WindowsOpen},
};

pub fn bottom_bar(ui: &mut egui::Ui, svg_logo: &mut RetainedImage, status: &mut Status) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("powered by ");
        ui.hyperlink_to("PrussianStudio", "https://github.com/crimsondamask");
        ui.with_layout(egui::Layout::right_to_left(), |ui| {
            ui.spacing_mut().item_spacing.x = 20.0;
            ui.label("v0.1");
            ui.horizontal(|ui| {
                let max_size = ui.available_size();
                svg_logo.show_max_size(
                    ui,
                    egui::Vec2 {
                        x: max_size.x,
                        y: 12.0,
                    },
                );
                ui.spacing_mut().item_spacing.x = 20.0;

                ui.spinner();
                ui.label(format!("{}", &status.websocket));
            });
        });
    });
}

pub fn top_bar(
    ui: &mut egui::Ui,
    frame: &mut eframe::Frame,
    windows_open: &mut WindowsOpen,
    device_windows_buffer: &mut DeviceWindowsBuffer,
    devices: &mut Vec<Device>,
    loggers: &mut Vec<Logger>,
    channel_windows_buffer: &mut ChannelWindowsBuffer,
    spawn_logging_thread: &mut bool,
    config_save_path: &mut PathBuf,
) {
    // The top panel is often a good place for a menu bar:
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
                let path = std::env::current_dir().unwrap();

                if let Some(res) = rfd::FileDialog::new()
                    .add_filter("ron", &["ron"])
                    .set_directory(&path)
                    .pick_file()
                {
                    let config_str = fs::read_to_string(res).expect("Couldn't read");
                    if let Ok(config) = ron::de::from_str(&config_str) {
                        let app_config: AppConfig = config;
                        *devices = app_config.devices;
                        *loggers = app_config.loggers;
                    }
                }
            }
            if ui.button("Save").clicked() {
                let config_path = std::env::current_dir().unwrap();
                if let Some(selected_path) = rfd::FileDialog::new()
                    .add_filter("ron", &["ron"])
                    .set_directory(&config_path)
                    .save_file()
                {
                    let pretty = PrettyConfig::new()
                        .depth_limit(7)
                        .separate_tuple_members(true)
                        .enumerate_arrays(true);
                    let app_config = AppConfig {
                        devices: devices.clone(),
                        loggers: loggers.clone(),
                    };

                    // Needs better error handling.
                    if let Ok(s) = to_string_pretty(&app_config, pretty) {
                        *config_save_path = selected_path.clone();
                        let mut f = fs::File::create(&selected_path).expect("Couldn't create file");
                        f.write_all(&s.as_bytes()).expect("Couldn't write");
                    }
                }
            }
            ui.separator();
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
            // if ui.button("Add new device").clicked() {
            //     windows_open.new_device = !windows_open.new_device;
            // }
            ui.menu_button("PLC", |ui| {
                if ui.button("Configure").clicked() {
                    device_windows_buffer.status = "".to_owned();
                    windows_open.plc = !windows_open.plc;
                    device_windows_buffer.name = devices[0].name.clone();
                    device_windows_buffer.scan_rate = devices[0].scan_rate.clone();
                    device_windows_buffer.config = devices[0].config.clone();
                }
                if ui.button("Channels").clicked() {
                    windows_open.device_channels = !windows_open.device_channels;
                    channel_windows_buffer.device_id = 0;
                }
            });
            ui.separator();
            ui.menu_button("Modbus Device", |ui| {
                if ui.button("Configure").clicked() {
                    windows_open.modbus_device = !windows_open.modbus_device;
                    device_windows_buffer.status = "".to_owned();
                    device_windows_buffer.name = devices[1].name.clone();
                    device_windows_buffer.scan_rate = devices[1].scan_rate.clone();
                    device_windows_buffer.config = devices[1].config.clone();
                }
                if ui.button("Channels").clicked() {
                    windows_open.device_channels = !windows_open.device_channels;
                    channel_windows_buffer.device_id = 1;
                }
            });
        });
        ui.menu_button("Logger", |ui| {
            if ui.button("Configure").clicked() {
                windows_open.logger_configure = !windows_open.logger_configure;
            }
        });
        ui.menu_button("Help", |ui| if ui.button("About").clicked() {});

        ui.with_layout(egui::Layout::right_to_left(), |ui| {
            ui.spacing_mut().item_spacing.x = 20.0;
            ui.label("");
            ui.add_enabled_ui(!*spawn_logging_thread, |ui| {
                if ui.button("▶ Start").clicked() {
                    *spawn_logging_thread = true;
                }
            });
        });
    });
}
