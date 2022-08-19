mod calculation;
mod channel;
mod modbus;

use std::{error::Error, fmt::Display};

pub use calculation::*;
pub use channel::*;
use serde::{Deserialize, Serialize};
use tokio_modbus::{client::sync::Context, prelude::*};

const NUM_CHANNELS: usize = 10;

// pub trait LoggerChannel {
//     fn id(&self) -> usize;
//     fn value(&self) -> f32;
// }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LoggerChannel {
    Channel(Channel),
    Calculation(Calculation),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Modbus,
    OpcServer,
    WebSocketServer,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DeviceConfig {
    pub address: String,
    pub port: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Device {
    pub name: String,
    pub device_type: DeviceType,
    pub config: DeviceConfig,
    pub channels: Vec<Channel>,
    pub status: String,
}

impl Device {
    pub fn new(
        name: String,
        device_type: DeviceType,
        config: DeviceConfig,
        channels: Vec<Channel>,
        status: String,
    ) -> Self {
        Self {
            name,
            device_type,
            config,
            channels,
            status,
        }
    }
    pub fn tcp_connect(&mut self) -> Result<Context, Box<dyn Error>> {
        let ip = &self.config.address;
        let port = &self.config.port;
        let socket = format!("{}:{}", ip, port).parse()?;

        let mut ctx = sync::tcp::connect(socket)?;
        // for channel in &mut self.channels {
        // channel.read_value(&mut ctx);
        // }

        Ok(ctx)
    }
}

impl Default for Device {
    fn default() -> Self {
        let mut channels = Vec::new();
        for i in 0..NUM_CHANNELS {
            let channel = Channel {
                id: i,
                ..Default::default()
            };
            channels.push(channel);
        }
        Device {
            name: "".to_owned(),
            device_type: DeviceType::Modbus,
            config: DeviceConfig {
                address: "127.0.0.1".to_owned(),
                port: 5502,
            },
            channels,
            status: "Initialized".to_owned(),
        }
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
