use crate::crossbeam::{CrossBeamChannel, CrossBeamSocketChannel, DeviceMsgBeam};
use crate::{app::TemplateApp, crossbeam::DeviceBeam};
use crossbeam_channel::unbounded;
use egui::{Context, InnerResponse};
use lib_device::*;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use tungstenite::stream::MaybeTlsStream;

use anyhow;
use serde::{Deserialize, Serialize};
use tungstenite::Message;
use tungstenite::{connect, WebSocket};
use url::Url;

#[derive(Serialize, Clone)]
struct DataSerialized {
    devices: Vec<Device>,
}
#[derive(Deserialize, Clone, PartialEq)]
pub struct JsonWriteChannel {
    pub device_id: usize,
    pub channel: usize,
    pub value: f32,
}

pub fn central_panel(ctx: &Context, app: &mut TemplateApp) -> InnerResponse<()> {
    let num_devices = app.devices.len();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.label("Monitor");
        ui.separator();

        // We check if the logging button has been pressed to spawn our threads.
        if app.spawn_logging_thread {
            app.spawn_logging_thread = !app.spawn_logging_thread;
            let devices_to_read = app.devices.clone();

            let (socket_s, socket_r): (
                crossbeam_channel::Sender<JsonWriteChannel>,
                crossbeam_channel::Receiver<JsonWriteChannel>,
            ) = unbounded();
            // We construct the channel for writing values from HMI.
            let socket_channel = CrossBeamSocketChannel {
                send: socket_s,
                receive: socket_r,
            };

            app.socket_channel = Some(socket_channel.clone());

            spawn_socket_recv(socket_channel);

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

                let device_beam = DeviceBeam {
                    read: Some(read_channel),
                    update: Some(update_channel),
                };

                app.device_msg_beam.push(device_msg_channel.clone());

                app.device_beam.push(device_beam.clone());

                spawn_device_thread(
                    devices_to_read.clone(),
                    device_beam.clone(),
                    device_msg_channel.clone(),
                    i,
                );
            }
        }

        egui::warn_if_debug_build(ui);

        // We try to receive any pending messages from all the threads.
        // Each thread has its own crossbeam channel.
        for i in 0..(num_devices) {
            if let Some(crossbeam) = &app.device_beam.iter().nth(i) {
                if let Some(devices) = crossbeam.read.clone() {
                    if let Ok(device_received) = devices.receive.try_recv() {
                        app.devices[i] = device_received[i].clone();
                    }
                }
            }
        }

        // We check if there is any new message from the HMI recv thread and update the corresponding device.
        write_channel_from_hmi(app);

        let data_to_serialize = DataSerialized {
            devices: app.devices.clone(),
        };
        // We send the data over the web socket to the HMI and update our status.
        app.status.websocket = match send_over_socket(&mut app.socket, &data_to_serialize) {
            Ok(_) => "Connected to WebSocket.".to_owned(),
            Err(e) => {
                app.socket = None;
                format!("ERROR: {}", e)
            }
        };

        if !app.socket.is_some() {
            if let Ok((socket, _)) = connect(Url::parse("wss://localhost:8080/socket").unwrap()) {
                app.socket = Some(socket);
            }
        }
    })
}

fn write_channel_from_hmi(app: &mut TemplateApp) {
    let socket_channel = &app.socket_channel;
    if let Some(socket_channel) = socket_channel {
        if let Ok(json_channel) = socket_channel.receive.try_recv() {
            match app.devices[json_channel.device_id].channels[json_channel.channel].access_type {
                AccessType::Write => {
                    app.devices[json_channel.device_id].channels[json_channel.channel].value =
                        json_channel.value;
                    println!("channel modified");
                    if let Some(device_beam) = app.device_beam.iter().nth(json_channel.device_id) {
                        if let Some(updated_channel) = device_beam.update.clone() {
                            if let Ok(_) = updated_channel.send.send(app.devices.to_vec()) {
                                println!("Sent the write update to the device worker.");
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
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

fn spawn_device_thread(
    mut devices_to_read: Vec<Device>,
    device_beam: DeviceBeam,
    device_msg_beam: DeviceMsgBeam,
    i: usize,
) {
    thread::spawn(move || {
        // We reset the device status.
        devices_to_read[i].status = "Initialized.".to_owned();
        // We spin the loop that reads data from the device.
        start_thread_loop(device_beam, device_msg_beam, devices_to_read, i)
    });
}

fn start_thread_loop(
    device_beam: DeviceBeam,
    device_msg_beam: DeviceMsgBeam,
    mut devices_to_read: Vec<Device>,
    i: usize,
) {
    loop {
        // This allows us to update the device config from the main thread.
        match devices_to_read[i].connect() {
            Ok(ctx) => {
                devices_to_read[i].status = "Connected.".to_owned();
                // This loop keeps on reading and updating device data.
                start_device_poll_loop(
                    &device_beam,
                    devices_to_read.clone(),
                    i,
                    &device_msg_beam,
                    ctx,
                )
            }
            Err(e) => devices_to_read[i].status = format!("Error: {}", e),
        }
    }
}

fn start_device_poll_loop(
    device_beam: &DeviceBeam,
    mut devices_to_read: Vec<Device>,
    i: usize,
    device_msg_beam: &DeviceMsgBeam,
    mut ctx: tokio_modbus::prelude::sync::Context,
) {
    loop {
        // We check if there is any update from the main thread.
        if let Some(crossbeam_channel) = device_beam.update.clone() {
            if let Ok(received_devices) = crossbeam_channel.receive.try_recv() {
                devices_to_read = received_devices.clone();
                devices_to_read[i].status = "Updated.".to_owned();
            }
        }

        // We check if there is any message to reconnect the device.
        // We update the ctx with the config wrapped in the received message.
        if let Ok(device_msg) = device_msg_beam.receive.try_recv() {
            match device_msg {
                DeviceMsg::Reconnect(config) => {
                    devices_to_read[i].config = config;
                    if let Ok(ctx_update) = devices_to_read[i].connect() {
                        ctx = ctx_update;
                    }
                }
            }
        }

        // We poll data from the device and send it to the main GUI thread.
        let channels = devices_to_read[i].channels.clone();
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
        devices_to_read[i].channels = channels_to_send;

        // Send the read data to the main GUI thread.
        if let Some(crossbeam_channel) = device_beam.read.clone() {
            if let Ok(_) = crossbeam_channel.send.send(devices_to_read.clone()) {}
        }
        // The thread sleeps.
        thread::sleep(Duration::from_secs(devices_to_read[i].scan_rate));
    }
}

pub fn spawn_socket_recv(socket_channel: CrossBeamSocketChannel) {
    thread::spawn(move || {
        if let Ok((mut socket, _)) = connect(Url::parse("wss://localhost:8080/socket").unwrap()) {
            loop {
                let msg = socket.read_message().expect("Error reading message");
                if let Ok(json_write_channel) = serde_json::from_str(msg.to_text().unwrap()) {
                    if socket_channel.send.send(json_write_channel).is_ok() {
                        println!("Channel serialized!");
                    }
                }
            }
        };
    });
}
