use crate::{
    app_threads::{spawn_device_thread, spawn_socket_recv, spawn_socket_write_msg},
    crossbeam::{CrossBeamChannel, CrossBeamSocketChannel, DeviceBeam, DeviceMsgBeam},
    fonts::*,
    setup_app::{setup_app_defaults, setup_visuals},
    status::Status,
    ui::{
        menu_bars::*,
        panels::{central_panel::*, left_panel::left_panel, right_panel::right_panel},
        windows::{device_windows::*, logger_windows::logger_config_window},
    },
    window::*,
};

use crossbeam_channel::unbounded;
use extras::RetainedImage;
pub use lib_device::Channel;
pub use lib_device::*;
pub use lib_logger::{parse_pattern, Logger, LoggerType};

use egui::Window;
use regex::Regex;
use serde::Serialize;
use std::net::TcpStream;
use tungstenite::{connect, stream::MaybeTlsStream};
use tungstenite::{Message, WebSocket};
use url::Url;

#[derive(Serialize, Clone)]
pub struct DataSerialized {
    pub devices: Vec<Device>,
}
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
    #[serde(skip)]
    pub allow_exit: bool,
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

    fn on_exit_event(&mut self) -> bool {
        self.windows_open.confirm_exit = true;
        self.allow_exit
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
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
            socket_channel,
            socket,
            re,
            svg_logo,
            ..
        } = self;

        let num_devices = devices.len();

        // We keep trying to reconnect to the websocket server if
        // there is no cnx established.

        if !socket.is_some() {
            if let Ok((socket_conn, _)) =
                connect(Url::parse("wss://localhost:8080/socket").unwrap())
            {
                *socket = Some(socket_conn);
            }
        }

        // We try to receive any pending messages from all the threads.
        // Each thread has its own crossbeam channel.
        // --------------------------------
        for i in 0..(num_devices) {
            if let Some(crossbeam) = device_beam.iter().nth(i) {
                if let Some(devices_received) = crossbeam.read.clone() {
                    if let Ok(device_received) = devices_received.receive.try_recv() {
                        devices[i] = device_received[i].clone();
                    }
                }
            }
        }
        // --------------------------------

        let data_to_serialize = DataSerialized {
            devices: devices.clone(),
        };
        // We send the data over the web socket to the HMI and update our status.
        status.websocket = match send_over_socket(socket, &data_to_serialize) {
            Ok(_) => "Connected to WebSocket.".to_owned(),
            Err(e) => {
                *socket = None;
                format!("ERROR: {}", e)
            }
        };

        // --------------------------------
        // We check if there is any write request from the HMI
        // This is achieved by checking the channel that the write websocket
        // communicates with.

        if let Some(socket_channel) = socket_channel {
            if let Ok(json_channel) = socket_channel.receive.try_recv() {
                match devices[json_channel.device_id].channels[json_channel.channel].access_type {
                    AccessType::Write => {
                        devices[json_channel.device_id].channels[json_channel.channel].value =
                            json_channel.value;
                        println!("channel modified");
                        if let Some(device_beam) = device_beam.iter().nth(json_channel.device_id) {
                            if let Some(updated_channel) = device_beam.update.clone() {
                                if let Ok(_) = updated_channel.send.send(devices.to_vec()) {
                                    println!("Sent the write update to the device worker.");
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // --------------------------------

        // We check if the start button has been pressed
        // Will change later.
        // --------------------------------
        if *spawn_logging_thread {
            *spawn_logging_thread = !*spawn_logging_thread;
            let devices_to_read = devices.clone();

            let (socket_s, socket_r): (
                crossbeam_channel::Sender<JsonWriteChannel>,
                crossbeam_channel::Receiver<JsonWriteChannel>,
            ) = unbounded();
            // We construct the channel for writing values from HMI.
            //let socket_channel_init = CrossBeamSocketChannel {
            //  send: socket_s,
            //receive: socket_r,
            // };

            //*socket_channel = Some(socket_channel_init.clone());

            //spawn_socket_recv(socket_channel_init);

            for i in 0..(num_devices) {
                let (device_msg_s, device_msg_r): (
                    crossbeam_channel::Sender<DeviceMsg>,
                    crossbeam_channel::Receiver<DeviceMsg>,
                ) = unbounded();
                let (read_s, read_r): (
                    crossbeam_channel::Sender<Vec<Device>>,
                    crossbeam_channel::Receiver<Vec<Device>>,
                ) = unbounded();
                let (update_s, update_r): (
                    crossbeam_channel::Sender<Vec<Device>>,
                    crossbeam_channel::Receiver<Vec<Device>>,
                ) = unbounded();

                let device_msg_channel = DeviceMsgBeam {
                    send: device_msg_s,
                    receive: device_msg_r,
                };
                let read_channel = CrossBeamChannel {
                    send: read_s.clone(),
                    receive: read_r.clone(),
                };
                let update_channel = CrossBeamChannel {
                    send: update_s,
                    receive: update_r.clone(),
                };

                let device_channel = DeviceBeam {
                    read: Some(read_channel),
                    update: Some(update_channel),
                };

                device_msg_beam.push(device_msg_channel.clone());

                device_beam.push(device_channel.clone());

                spawn_device_thread(
                    devices_to_read.clone(),
                    device_channel.clone(),
                    device_msg_channel.clone(),
                    i,
                );
            }
            spawn_socket_write_msg(device_msg_beam.to_vec());
        }
        // --------------------------------

        // --------------------------------
        // Drawing the UI

        if windows_open.confirm_exit {
            egui::Window::new("Confirm exit")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Are you sure you want to quit the application?");
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            windows_open.confirm_exit = false;
                        }

                        if ui.button("Yes").clicked() {
                            self.allow_exit = true;
                            frame.quit();
                        }
                    });
                });
        }
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

            write_channel_value_ui(windows_open, ctx, channel_windows_buffer, device_msg_beam);

            preferences_ui(windows_open, ctx);

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

fn write_channel_value_ui(
    windows_open: &mut WindowsOpen,
    ctx: &egui::Context,
    channel_windows_buffer: &mut ChannelWindowsBuffer,
    //devices: &mut Vec<Device>,
    device_msg_beam: &mut Vec<DeviceMsgBeam>,
) {
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
                        //devices[channel_windows_buffer.device_id].channels
                        //  [channel_windows_buffer.selected_channel.id]
                        //.value = value;

                        if let Some(device_msg_beam) =
                            device_msg_beam.iter().nth(channel_windows_buffer.device_id)
                        {
                            let channel_to_write = JsonWriteChannel {
                                device_id: channel_windows_buffer.device_id,
                                channel: channel_windows_buffer.selected_channel.id,
                                value,
                            };
                            if let Ok(_) = device_msg_beam
                                .send
                                .send(DeviceMsg::WriteChannel(channel_to_write))
                            {
                            }
                        }
                    }
                }
            });
        });
}

fn preferences_ui(windows_open: &mut WindowsOpen, ctx: &egui::Context) {
    Window::new("Preferences")
        .open(&mut windows_open.preferences)
        .show(ctx, |ui| {
            ctx.settings_ui(ui);
        });
}

fn send_over_socket(
    socket: &mut Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    data: &DataSerialized,
) -> anyhow::Result<()> {
    let json = serde_json::to_string(data)?;
    if let Some(socket) = socket {
        socket.write_message(Message::Text(json))?;
        Ok(())
    } else {
        anyhow::bail!("There is no socket connected!")
    }
}
