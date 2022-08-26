use crate::{
    crossbeam::DeviceBeam,
    fonts::*,
    panels::{central_panel::central_panel, left_panel::left_panel, right_panel::right_panel},
    window::{DeviceType, *},
};

use extras::RetainedImage;
pub use lib_device::Channel;
pub use lib_device::*;
pub use lib_logger::{parse_pattern, Logger, LoggerType};

use egui::{Button, Grid, Slider};
use egui::{Color32, ComboBox, Rounding, Window};
use regex::Regex;
use rfd::FileDialog;
use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, WebSocket};
use url::Url;
const NUM_CHANNELS: usize = 10;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)]
    pub test_str: String,
    pub logger_window_buffer: LoggerWindowBuffer,
    pub device_windows_buffer: DeviceWindowsBuffer,
    #[serde(skip)]
    pub channel_windows_buffer: ChannelWindowsBuffer,
    #[serde(skip)]
    pub value: f32,
    #[serde(skip)]
    pub windows_open: WindowsOpen,
    pub devices: Vec<Device>,
    pub loggers: Vec<Logger>,
    #[serde(skip)]
    pub device_beam: Vec<DeviceBeam>,
    #[serde(skip)]
    pub spawn_logging_thread: bool,
    #[serde(skip)]
    pub re: (Regex, Regex),
    #[serde(skip)]
    pub socket: WebSocket<MaybeTlsStream<TcpStream>>,
    #[serde(skip)]
    pub svg_logo: RetainedImage,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let (socket, _) =
            connect(Url::parse("ws://localhost:12345/socket").unwrap()).expect("Can't connect.");
        Self {
            // Example stuff:
            test_str: String::new(),
            logger_window_buffer: LoggerWindowBuffer::default(),
            device_windows_buffer: DeviceWindowsBuffer::default(),
            channel_windows_buffer: ChannelWindowsBuffer {
                channel_write_value: vec![String::new(); NUM_CHANNELS],
                device_id: 0,
                ..Default::default()
            },
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
            loggers: Vec::new(),
            device_beam: Vec::new(),
            spawn_logging_thread: false,
            re: (
                Regex::new(r"CH+(?:([0-9]+))").unwrap(),
                Regex::new(r"EVAL+(?:([0-9]+))").unwrap(),
            ),
            socket,
            svg_logo: RetainedImage::from_svg_bytes("svg_logo.svg", include_bytes!("svg_logo.svg"))
                .unwrap(),
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
            test_str,
            logger_window_buffer,
            device_windows_buffer,
            channel_windows_buffer,
            windows_open,
            devices,
            loggers,
            device_beam,
            re,
            svg_logo,
            ..
        } = self;

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
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
            });
            Window::new("PLC Channels")
                .open(&mut windows_open.device_channels)
                .scroll2([false, true])
                .show(ctx, |ui| {
                    Grid::new("Channel List")
                        .striped(true)
                        .num_columns(8)
                        .min_col_width(160.0)
                        .show(ui, |ui| {
                            if let Some(device) =
                                &devices.iter().nth(channel_windows_buffer.device_id)
                            {
                                ui.label(format!("{}", &device));
                                ui.label("Device status:");
                                ui.label(format!("{}", &device.status));

                                ui.end_row();
                                ui.separator();
                                ui.end_row();
                                ui.label("Channel");
                                ui.label("Value");
                                ui.label("Alarm");
                                ui.label("Tag");
                                ui.label("Value type");
                                ui.label("Access");
                                ui.label("Address");
                                ui.label("Status");
                                ui.end_row();
                                for _ in 0..8 {
                                    ui.separator();
                                }
                                ui.end_row();
                                for channel in device.channels.clone() {
                                    let button =
                                        Button::new(format!("CH{}", channel.id)).frame(true);
                                    if ui.add(button).clicked() {
                                        channel_windows_buffer.selected_channel = channel.clone();
                                        windows_open.channel_config = !windows_open.channel_config;
                                        channel_windows_buffer.edited_channel =
                                            channel_windows_buffer.selected_channel.clone();
                                    }
                                    // ui.label(format!("CH{}", channel.id));
                                    match channel.access_type {
                                        AccessType::Write => {
                                            ui.horizontal(|ui| {
                                                ui.label(format!("{:.2}", channel.value));
                                                if ui.button("Write").clicked() {
                                                    channel_windows_buffer.selected_channel =
                                                        channel.clone();

                                                    windows_open.channel_write_value =
                                                        !windows_open.channel_write_value;
                                                }
                                            });
                                        }
                                        AccessType::Read => {
                                            ui.label(format!("{:.2}", channel.value));
                                        }
                                    };
                                    let mut alarm = "";
                                    if channel.alarm.low.active {
                                        alarm = "LOW ALARM";
                                    }

                                    if channel.alarm.high.active {
                                        alarm = "HIGH ALARM";
                                    }
                                    if channel.alarm.low.active && channel.alarm.high.active {
                                        alarm = "LOW ALARM/HIGH ALARM";
                                    }

                                    ui.colored_label(Color32::RED, alarm);

                                    ui.label(format!("{}", channel.tag));
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
                            channel_windows_buffer.edited_channel.id =
                                channel_windows_buffer.selected_channel.id;
                            ui.label("Tag");
                            ui.text_edit_singleline(&mut channel_windows_buffer.edited_channel.tag);
                            ui.end_row();
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
                            ui.label("Low alarm:");
                            ui.label("High alarm:");
                            ui.end_row();
                            ui.checkbox(
                                &mut channel_windows_buffer.edited_channel.alarm.low.enabled,
                                "Enabled",
                            );
                            ui.checkbox(
                                &mut channel_windows_buffer.edited_channel.alarm.high.enabled,
                                "Enabled",
                            );
                            ui.end_row();
                            ui.add_enabled_ui(
                                channel_windows_buffer.edited_channel.alarm.low.enabled,
                                |ui| {
                                    ui.add(
                                        Slider::new(
                                            &mut channel_windows_buffer
                                                .edited_channel
                                                .alarm
                                                .low
                                                .setpoint,
                                            0.0..=1000.0,
                                        )
                                        .text(""),
                                    );
                                },
                            );
                            ui.add_enabled_ui(
                                channel_windows_buffer.edited_channel.alarm.high.enabled,
                                |ui| {
                                    ui.add(
                                        Slider::new(
                                            &mut channel_windows_buffer
                                                .edited_channel
                                                .alarm
                                                .high
                                                .setpoint,
                                            0.0..=1000.0,
                                        )
                                        .text(""),
                                    );
                                },
                            );

                            ui.end_row();
                        });
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Save").clicked() {
                            devices[channel_windows_buffer.device_id].channels
                                [channel_windows_buffer.selected_channel.id] =
                                channel_windows_buffer.edited_channel.clone();
                            if let Some(device_beam) =
                                device_beam.iter().nth(channel_windows_buffer.device_id)
                            {
                                if let Some(updated_channel) = device_beam.update.clone() {
                                    if updated_channel.send.send(devices.to_vec()).is_ok() {}
                                }
                            }
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
                                        devices[0].name = device_windows_buffer.name.clone();
                                        devices[0].config = config;
                                        devices[0].scan_rate = device_windows_buffer.scan_rate;
                                        device_windows_buffer.status =
                                            "Device configuration saved successfully!".to_owned();
                                        if let Some(device_beam) = device_beam.iter().nth(0) {
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
                                        devices[1].config = config;
                                        device_windows_buffer.status =
                                            "Device configuration saved successfully!".to_owned();
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
            Window::new("Add Device")
                .open(&mut windows_open.new_device)
                .show(ctx, |ui| {});
        });

        right_panel(ctx, &self);
        left_panel(ctx, self);
        central_panel(ctx, self);
    }
}
