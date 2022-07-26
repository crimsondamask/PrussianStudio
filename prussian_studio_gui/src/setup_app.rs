use std::path::PathBuf;

use egui::{Color32, Rounding};
use extras::RetainedImage;
use lib_device::Device;
use regex::Regex;
use rhai::Engine;
use tungstenite::connect;
use url::Url;

use crate::{
    app::URL,
    status::Status,
    window::{ChannelWindowsBuffer, DeviceWindowsBuffer, LoggerWindowBuffer, WindowsOpen},
    TemplateApp,
};
const NUM_CHANNELS: usize = 20;

pub fn setup_app_defaults() -> TemplateApp {
    let socket = match connect(Url::parse(URL).unwrap()) {
        Ok((socket, _)) => Some(socket),
        Err(_) => None,
    };
    TemplateApp {
        // Example stuff:
        rhai_engine: Engine::new(),
        status: Status::default(),
        logger_window_buffer: LoggerWindowBuffer::default(),
        device_windows_buffer: DeviceWindowsBuffer::default(),
        channel_windows_buffer: ChannelWindowsBuffer {
            channel_write_value: vec![String::new(); NUM_CHANNELS],
            device_id: 0,
            ..Default::default()
        },
        windows_open: WindowsOpen::default(),
        devices: vec![
            Device::initialize(0, "PLC".to_owned()),
            Device::initialize(1, "Modbus device".to_owned()),
        ],
        loggers: Vec::new(),
        device_beam: Vec::new(),
        device_msg_beam: Vec::new(),
        socket_channel: None,
        spawn_logging_thread: false,
        re: (
            Regex::new(r"CH+(?:([0-9]+))").unwrap(),
            Regex::new(r"EVAL+(?:([0-9]+))").unwrap(),
        ),
        socket,
        svg_logo: RetainedImage::from_svg_bytes("svg_logo.svg", include_bytes!("svg_logo.svg"))
            .unwrap(),
        allow_exit: false,
        config_save_path: PathBuf::from("."),
    }
}

pub fn setup_visuals(cc: &eframe::CreationContext<'_>) {
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
}
