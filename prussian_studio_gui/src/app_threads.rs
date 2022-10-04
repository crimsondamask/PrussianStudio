use crate::crossbeam::{CrossBeamSocketChannel, DeviceBeam, DeviceMsgBeam};
use lib_device::{get_register_list, AccessType, Device, DeviceMsg};
use std::{thread, time::Duration};
use tungstenite::connect;
use url::Url;

pub fn spawn_device_thread(
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

pub fn start_thread_loop(
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

pub fn start_device_poll_loop(
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

        let (_, reg_list) = get_register_list(&devices_to_read[i]);
        println!("{:?}", reg_list);
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
                if let Ok(msg) = socket.read_message() {
                    if let Ok(json_write_channel) = serde_json::from_str(msg.to_text().unwrap()) {
                        if socket_channel.send.send(json_write_channel).is_ok() {
                            println!("Channel serialized!");
                        }
                    }
                } else {
                    if let Ok((socket_reconn, _)) =
                        connect(Url::parse("wss://localhost:8080/socket").unwrap())
                    {
                        socket = socket_reconn;
                    }
                }
            }
        };
    });
}
