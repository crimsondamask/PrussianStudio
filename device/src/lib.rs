mod calculation;
mod channel;
mod config;
mod logger_channel;
mod modbus;

use std::{error::Error, fmt::Display};

pub use calculation::*;
pub use channel::*;
pub use config::*;
pub use logger_channel::*;
use serde::{Deserialize, Serialize};
use tokio_modbus::{client::sync::Context, prelude::*};
use tokio_serial;

const NUM_CHANNELS: usize = 10;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Modbus,
    OpcServer,
    WebSocketServer,
}

// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
// pub struct DeviceConfig {
//     pub address: String,
//     pub port: usize,
// }

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
    pub fn connect(&mut self) -> Result<Context, Box<dyn Error>> {
        let ctx = match &self.config {
            DeviceConfig::Tcp(config) => {
                let address = config.address.to_owned();
                let port = config.port;
                let socket = format!("{}:{}", address, port).parse()?;
                sync::tcp::connect(socket)?
            }
            DeviceConfig::Serial(config) => {
                let path = config.com_port.to_owned();
                let baudrate = config.baudrate;
                let slave = Slave(config.slave);
                let builder = tokio_serial::new(path, baudrate);
                sync::rtu::connect_slave(&builder, slave)?
            }
        };
        // for channel in &mut self.channels {
        // channel.read_value(&mut ctx);
        // }

        Ok(ctx)
    }
}

impl Default for Device {
    fn default() -> Self {
        let mut channels = Vec::new();
        let config = DeviceConfig::Tcp(TcpConfig {
            address: "127.0.0.1".to_owned(),
            port: 502,
        });
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
            config,
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
