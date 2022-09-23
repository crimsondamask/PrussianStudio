use egui::{Button, Color32, Grid, Slider, Window};
use lib_device::*;

use crate::{
    crossbeam::{DeviceBeam, DeviceMsgBeam},
    window::{ChannelWindowsBuffer, DeviceType, DeviceWindowsBuffer, WindowsOpen},
};

pub fn plc_config_window(
    windows_open: &mut WindowsOpen,
    ctx: &egui::Context,
    device_windows_buffer: &mut DeviceWindowsBuffer,
    devices: &mut Vec<Device>,
    device_msg_beam: &mut Vec<DeviceMsgBeam>,
    device_beam: &mut Vec<DeviceBeam>,
) {
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
                                devices[0].config = config.clone();
                                devices[0].scan_rate = device_windows_buffer.scan_rate;
                                device_windows_buffer.status =
                                    "Device configuration saved successfully!".to_owned();
                                if let Some(device_msg) = device_msg_beam.iter().nth(0) {
                                    if device_msg.send.send(DeviceMsg::Reconnect(config)).is_ok() {
                                        println!("config sent!");
                                    }
                                }
                                if let Some(device_beam) = device_beam.iter().nth(0) {
                                    if let Some(updated_device) = device_beam.update.clone() {
                                        if updated_device.send.send(devices.to_vec()).is_ok() {}
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
}

pub fn plc_channels_window(
    windows_open: &mut WindowsOpen,
    ctx: &egui::Context,
    devices: &mut Vec<Device>,
    channel_windows_buffer: &mut ChannelWindowsBuffer,
) {
    Window::new("PLC Channels")
        .open(&mut windows_open.device_channels)
        .scroll2([true, true])
        .show(ctx, |ui| {
            Grid::new("Channel List")
                .striped(true)
                .num_columns(9)
                .min_col_width(160.0)
                .show(ui, |ui| {
                    if let Some(device) = &devices.iter().nth(channel_windows_buffer.device_id) {
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
                        ui.label("Device");
                        ui.label("Status");
                        ui.end_row();
                        for _ in 0..9 {
                            ui.separator();
                        }
                        ui.end_row();
                        for channel in device.channels.clone() {
                            let button = Button::new(format!("CH{}", channel.id)).frame(true);
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
                            ui.label(format!("{}", channel.device_id));
                            ui.label(format!("{}", channel.status));
                            // if ui.small_button("Configure").clicked() {}
                            ui.end_row();
                        }
                    }
                });
        });
}
