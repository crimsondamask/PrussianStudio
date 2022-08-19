use crate::app::TemplateApp;
// use crate::window::*;
use crossbeam_channel::unbounded;
use egui::{ComboBox, Context, InnerResponse};
use lib_device::*;
use rand::Rng;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use tungstenite::protocol::WebSocketContext;
use tungstenite::stream::MaybeTlsStream;

use serde::{Deserialize, Serialize};
use serde_json::Result;
use tungstenite::{connect, Message, WebSocket};
use url::Url;
// use tokio_modbus::prelude::*;

#[derive(Serialize)]
struct DataSerialized {
    devices: Vec<Device>,
}

pub fn central_panel(ctx: &Context, app: &mut TemplateApp) -> InnerResponse<()> {
    // thread::spawn(move || loop {
    //     thread::sleep(Duration::from_secs(5));
    // });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.label("Monitor");
        ui.separator();
        // if ui.button("Start logging").clicked() {
        // app.spawn_logging_thread = !app.spawn_logging_thread;
        // }

        if app.spawn_logging_thread {
            app.spawn_logging_thread = !app.spawn_logging_thread;
            // let (tx, rx): (Sender<Vec<Device>>, Receiver<Vec<Device>>) = mpsc::channel();
            let (read_s, read_r): (
                crossbeam_channel::Sender<Vec<Device>>,
                crossbeam_channel::Receiver<Vec<Device>>,
            ) = unbounded();
            let (update_s, update_r): (
                crossbeam_channel::Sender<Vec<Device>>,
                crossbeam_channel::Receiver<Vec<Device>>,
            ) = unbounded();

            app.read_channel = Some((read_s.clone(), read_r));
            app.update_channel = Some((update_s, update_r.clone()));

            let mut devices_to_read = app.devices.clone();
            let mut data_to_serialize = DataSerialized {
                devices: devices_to_read.clone(),
            };

            thread::spawn(move || {
                let (mut socket, _) = connect(Url::parse("ws://localhost:12345/socket").unwrap())
                    .expect("Can't connect.");

                match devices_to_read[0].tcp_connect() {
                    Ok(mut ctx) => {
                        println!("Connected.");
                        devices_to_read[0].status = "Connected.".to_owned();
                        loop {
                            thread::sleep(Duration::from_secs(1));
                            if let Ok(received_devices) = update_r.try_recv() {
                                devices_to_read = received_devices.clone();
                            }

                            let channels = devices_to_read[0].channels.clone();
                            let mut channels_to_send = Vec::with_capacity(channels.len());
                            for mut channel in channels.clone() {
                                match channel.access_type {
                                    AccessType::Read => {
                                        channel.read_value(&mut ctx);
                                    }
                                    AccessType::Write => {
                                        channel.write_value(&mut ctx);
                                        // We need to read the value after the write to see it updated.

                                        channel.read_value(&mut ctx);
                                    }
                                }

                                channels_to_send.push(channel);
                            }
                            devices_to_read[0].channels = channels_to_send;

                            // Send the read data to the main GUI thread.
                            if let Ok(_) = read_s.send(devices_to_read.clone()) {}

                            // Updating the serialized data and sending it over the websocket server.
                            data_to_serialize.devices = devices_to_read.clone();
                            send_over_socket(&mut socket, &data_to_serialize);
                        }
                    }
                    Err(e) => devices_to_read[0].status = format!("Error: {}", e),
                }
            });
        }

        egui::warn_if_debug_build(ui);
        ComboBox::from_label("Device")
            .selected_text(format!("{}", app.channel_windows_buffer.selected_device))
            .show_ui(ui, |ui| {
                for device in &app.devices {
                    ui.selectable_value(
                        &mut app.channel_windows_buffer.selected_device,
                        device.clone(),
                        format!("{}", device),
                    );
                }
            });
        if let Some(devices) = &app.read_channel {
            if let Ok(device_received) = devices.1.try_recv() {
                app.devices = device_received;
            }
        }
        // ui.label(format!("{}", &app.channel_windows_buffer.selected_device));
    })
}

fn send_over_socket(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, data: &DataSerialized) {
    if let Ok(json) = serde_json::to_string(data) {
        if let Ok(_) = socket.write_message(Message::Text(json)) {}
    }
}

fn spawn_device_thread(devices_to_read: Vec<Device>, data_to_serialize: DataSerialized) {}
