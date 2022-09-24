use crate::{
    crossbeam::{CrossBeamSocketChannel, DeviceBeam, DeviceMsgBeam},
    fonts::*,
    setup_app::{setup_app_defaults, setup_visuals},
    status::Status,
    ui::panels::{central_panel::*, left_panel::left_panel, right_panel::right_panel},
    ui::{
        menu_bars::*,
        windows::{device_windows::*, logger_windows::logger_config_window},
    },
    window::*,
};

use extras::RetainedImage;
pub use lib_device::Channel;
pub use lib_device::*;
pub use lib_logger::{parse_pattern, Logger, LoggerType};

use egui::Window;
use regex::Regex;
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

            logger_config_window(windows_open, ctx, logger_window_buffer, re, loggers);

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

            device_config_window(
                windows_open,
                ctx,
                device_windows_buffer,
                devices,
                device_msg_beam,
                device_beam,
            );
        });

        right_panel(ctx, &self);
        left_panel(ctx, self);
        central_panel(ctx, self);
    }
}
