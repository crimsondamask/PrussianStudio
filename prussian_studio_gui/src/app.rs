use crate::{
    crossbeam::{CrossBeamSocketChannel, DeviceBeam, DeviceMsgBeam},
    fonts::*,
    panels::{central_panel::*, left_panel::left_panel, right_panel::right_panel},
    setup_app::{setup_app_defaults, setup_visuals},
    status::Status,
    ui::{menu_bars::*, panels::*, windows::device_windows::*},
    window::{DeviceType, *},
};

use extras::RetainedImage;
pub use lib_device::Channel;
pub use lib_device::*;
pub use lib_logger::{parse_pattern, Logger, LoggerType};

use egui::{Color32, ComboBox, Window};
use egui::{Grid, Slider};
use regex::Regex;
use rfd::FileDialog;
use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)]
    pub status: Status,
    pub logger_window_buffer: LoggerWindowBuffer,
    pub device_windows_buffer: DeviceWindowsBuffer,
    #[serde(skip)]
    pub channel_windows_buffer: ChannelWindowsBuffer,
    #[serde(skip)]
    pub windows_open: WindowsOpen,
    pub devices: Vec<Device>,
    pub loggers: Vec<Logger>,
    // We use this beam to send and receive Device data and config.
    #[serde(skip)]
    pub device_beam: Vec<DeviceBeam>,
    // We use this beam to send device reconnect requests.
    #[serde(skip)]
    pub device_msg_beam: Vec<DeviceMsgBeam>,
    // The crossbeam channel that we receive any write requests from the HMI on.
    #[serde(skip)]
    pub socket_channel: Option<CrossBeamSocketChannel>,
    // ---------------------
    #[serde(skip)]
    pub spawn_logging_thread: bool,
    #[serde(skip)]
    pub re: (Regex, Regex),
    #[serde(skip)]
    pub socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    #[serde(skip)]
    pub svg_logo: RetainedImage,
}

impl Default for TemplateApp {
    fn default() -> Self {
        setup_app_defaults()
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        setup_custom_fonts(&cc.egui_ctx);

        setup_visuals(cc);

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
            status,
            logger_window_buffer,
            device_windows_buffer,
            channel_windows_buffer,
            windows_open,
            devices,
            loggers,
            device_beam,
            device_msg_beam,
            spawn_logging_thread,
            re,
            svg_logo,
            ..
        } = self;

        // -------------------------------
        // We check if we received a msg from the HMI and we update the channels.
        // -------------------------------

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            bottom_bar(ui, svg_logo, status);
        });
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            top_bar(
                ui,
                frame,
                windows_open,
                device_windows_buffer,
                devices,
                channel_windows_buffer,
                spawn_logging_thread,
            );
            plc_channels_window(windows_open, ctx, devices, channel_windows_buffer);
            Window::new("Write Value")
                .open(&mut windows_open.channel_write_value)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(
                            &mut channel_windows_buffer.channel_write_value
                                [channel_windows_buffer.selected_channel.id],
                        );
                        if ui.button("Write").clicked() {
                            if let Ok(value) = channel_windows_buffer.channel_write_value
                                [channel_windows_buffer.selected_channel.id]
                                .parse::<f32>()
                            {
                                devices[channel_windows_buffer.device_id].channels
                                    [channel_windows_buffer.selected_channel.id]
                                    .value = value;

                                if let Some(device_beam) =
                                    device_beam.iter().nth(channel_windows_buffer.device_id)
                                {
                                    if let Some(updated_channel) = device_beam.update.clone() {
                                        if let Ok(_) = updated_channel.send.send(devices.to_vec()) {
                                        }
                                    }
                                }
                            }
                        }
                    });
                });

            Window::new("Preferences")
                .open(&mut windows_open.preferences)
                .show(ctx, |ui| {
                    ctx.settings_ui(ui);
                });
            Window::new("Configure Logger")
                .open(&mut windows_open.logger_configure)
                .scroll2([true, true])
                .show(ctx, |ui| {
                    Grid::new("Logger List")
                        .striped(true)
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Logger name:");
                            ui.add(egui::TextEdit::singleline(
                                &mut logger_window_buffer.logger_name,
                            ));
                            ui.end_row();
                            ui.label("Type");
                            ComboBox::from_label("")
                                .selected_text(format!("{}", logger_window_buffer.logger_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut logger_window_buffer.logger_type,
                                        LoggerType::DataBase,
                                        "Database",
                                    );
                                    ui.selectable_value(
                                        &mut logger_window_buffer.logger_type,
                                        LoggerType::TextFile,
                                        "Text File",
                                    );
                                });
                            ui.end_row();
                            ui.label("Log rate:");
                            ui.add(
                                Slider::new(&mut logger_window_buffer.log_rate, 1..=900)
                                    .text("Seconds"),
                            );
                            ui.end_row();
                            ui.label("Channel pattern:");
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut logger_window_buffer.channel_pattern.pattern,
                                )
                                .hint_text("Ex: CH1-CH7, EVAL10-EVAL20"),
                            );
                            ui.end_row();
                            match parse_pattern(
                                &logger_window_buffer.channel_pattern,
                                (&re.0, &re.1),
                            ) {
                                Ok(channels) => {
                                    ui.colored_label(
                                        Color32::DARK_GREEN,
                                        format!("{} channels will be logged.", &channels.len()),
                                    );
                                }
                                Err(e) => {
                                    ui.colored_label(Color32::RED, format!("{}", e));
                                }
                            }
                            ui.end_row();
                            if ui.button("Path").clicked() {
                                if let Some(path) = FileDialog::new().set_directory(".").save_file()
                                {
                                    logger_window_buffer.path = path;
                                }
                            }
                            ui.label(format!("{}", &logger_window_buffer.path.to_str().unwrap()));
                            ui.end_row();
                            ui.label("Logger list");
                            ui.end_row();
                            for logger in loggers.iter() {
                                ui.label("Name:");
                                ui.label(format!("{}", &logger.name));
                                ui.end_row();
                                ui.label("Number of channels:");
                                ui.label(format!("{}", &logger.channels.len()));
                                ui.end_row();
                                ui.label("Type:");
                                ui.label(format!("{}", &logger.logger_type));
                                ui.end_row();
                                ui.label("Logging rate:");
                                ui.label(format!("{} seconds", &logger.log_rate));
                                ui.end_row();
                                ui.separator();
                                ui.end_row();
                            }
                            ui.vertical_centered_justified(|ui| {
                                if ui.button("Save").clicked() {
                                    let logger = Logger::new(
                                        logger_window_buffer.logger_name.clone(),
                                        logger_window_buffer.logger_type.clone(),
                                        &mut logger_window_buffer.channel_pattern,
                                        logger_window_buffer.path.clone(),
                                        logger_window_buffer.log_rate,
                                        false,
                                        (&re.0, &re.1),
                                    );

                                    if let Ok(logger) = logger {
                                        loggers.push(logger);
                                    }
                                }
                            });
                        });
                });
            channel_config_window(
                windows_open,
                ctx,
                channel_windows_buffer,
                devices,
                device_beam,
            );
            plc_config_window(
                windows_open,
                ctx,
                device_windows_buffer,
                devices,
                device_msg_beam,
                device_beam,
            );
            Window::new("Modbus Device")
                .open(&mut windows_open.modbus_device)
                .scroll2([false, true])
                .show(ctx, |ui| {
                    ui.label("Configuration");
                    ui.separator();
                    egui::Grid::new("add_device").num_columns(2).show(ui, |ui| {
                        ui.label("Device name:");
                        ui.text_edit_singleline(&mut device_windows_buffer.name);
                        ui.end_row();
                        match device_windows_buffer.device_type {
                            DeviceType::Tcp => {
                                ui.label("IP address:");
                                ui.text_edit_singleline(&mut device_windows_buffer.address);
                                ui.end_row();
                                ui.label("Port:");
                                ui.text_edit_singleline(&mut device_windows_buffer.port);
                                ui.end_row();
                            }
                            DeviceType::Serial => {
                                ui.label("COM port:");
                                ui.text_edit_singleline(&mut device_windows_buffer.path);
                                ui.end_row();
                                ui.label("Baudrate:");
                                ui.text_edit_singleline(&mut device_windows_buffer.baudrate);
                                ui.end_row();
                                ui.label("Slave:");
                                ui.text_edit_singleline(&mut device_windows_buffer.slave);
                                ui.end_row();
                            }
                        }
                        ui.label("Scan rate:");
                        ui.add(Slider::new(&mut device_windows_buffer.scan_rate, 0..=60).text(""));
                        ui.end_row();
                    });
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Save").clicked() {
                            match device_windows_buffer.device_type {
                                DeviceType::Tcp => {
                                    if let Ok(port) = device_windows_buffer.port.parse::<usize>() {
                                        let config = DeviceConfig::Tcp(TcpConfig {
                                            address: device_windows_buffer.address.to_owned(),
                                            port,
                                        });
                                        devices[1].name = device_windows_buffer.name.clone();
                                        devices[1].config = config.clone();
                                        device_windows_buffer.status =
                                            "Device configuration saved successfully!".to_owned();
                                        if let Some(device_msg) = device_msg_beam.iter().nth(1) {
                                            if device_msg
                                                .send
                                                .send(DeviceMsg::Reconnect(config))
                                                .is_ok()
                                            {
                                            }
                                        }
                                        if let Some(device_beam) = device_beam.iter().nth(1) {
                                            if let Some(updated_device) = device_beam.update.clone()
                                            {
                                                if updated_device
                                                    .send
                                                    .send(devices.to_vec())
                                                    .is_ok()
                                                {
                                                }
                                            }
                                        }
                                    } else {
                                        device_windows_buffer.status = "Error!".to_owned();
                                    }
                                }
                                DeviceType::Serial => {}
                            }
                        }
                        ui.label(device_windows_buffer.status.to_owned());
                    });
                });
        });

        right_panel(ctx, &self);
        left_panel(ctx, self);
        central_panel(ctx, self);
    }
}
