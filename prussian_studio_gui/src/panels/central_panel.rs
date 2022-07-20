use crate::app::TemplateApp;
// use crate::window::*;
use egui::{ComboBox, Context, InnerResponse};
use lib_device::*;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

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
            thread::spawn(move || loop {
                thread::sleep(Duration::from_secs(5));
                let devices = vec![Device {
                    name: "PLC Received".to_owned(),
                    channels: vec![
                        Channel {
                            value: 5.0,
                            ..Default::default()
                        };
                        10
                    ],
                    ..Default::default()
                }];
                if let Ok(_) = tx.send(devices) {}
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
