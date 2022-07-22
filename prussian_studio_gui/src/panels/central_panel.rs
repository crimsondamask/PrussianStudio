use crate::app::TemplateApp;
// use crate::window::*;
use egui::{ComboBox, Context, InnerResponse};
use lib_device::*;
use rand::Rng;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
// use tokio_modbus::prelude::*;

pub fn central_panel(ctx: &Context, app: &mut TemplateApp) -> InnerResponse<()> {
    // thread::spawn(move || loop {
    //     thread::sleep(Duration::from_secs(5));
    // });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.label("Monitor");
        ui.separator();
        ui.toggle_value(&mut app.spawn_logging_thread, "Logging");
        // if ui.button("Start logging").clicked() {
        // app.spawn_logging_thread = !app.spawn_logging_thread;
        // }

        if !app.spawn_logging_thread {
            let (tx, rx): (Sender<Vec<Device>>, Receiver<Vec<Device>>) = mpsc::channel();
            app.mpsc_channel = Some((tx.clone(), rx));
            let mut devices_to_read = app.devices.clone();
            thread::spawn(move || {
                match devices_to_read[0].tcp_connect() {
                    Ok(mut ctx) => {
                        devices_to_read[0].status = "Connected.".to_owned();
                        loop {
                            thread::sleep(Duration::from_secs(1));

                            let channels = devices_to_read[0].channels.clone();
                            let mut channels_to_send = Vec::with_capacity(channels.len());
                            for mut channel in channels.clone() {
                                // channel.value = rand::thread_rng().gen_range(0.0..10.0);
                                channel.read_value(&mut ctx);
                                channels_to_send.push(channel);
                            }
                            // let devices = vec![Device {
                            //     name: "PLC".to_owned(),
                            //     channels,
                            //     ..Default::default()
                            // }];
                            devices_to_read[0].channels = channels_to_send;
                        }
                    }
                    Err(e) => devices_to_read[0].status = format!("Error: {}", e),
                }
                if let Ok(_) = tx.send(devices_to_read.clone()) {}
            });
        }
        // The central panel the region left after adding TopPanel's and SidePanel's

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
        if let Some(devices) = &app.mpsc_channel {
            if let Ok(device_received) = devices.1.try_recv() {
                app.devices = device_received;
            }
        }
        ui.label(format!("{}", &app.channel_windows_buffer.selected_device));
        if app.spawn_logging_thread {
            ui.label("hello");
        }
    })
}
