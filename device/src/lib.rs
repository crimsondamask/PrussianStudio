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
    pub id: usize,
    pub name: String,
    pub device_type: DeviceType,
    pub config: DeviceConfig,
    pub channels: Vec<Channel>,
    pub scan_rate: u64,
    pub status: String,
}

impl Device {
    pub fn new(
        id: usize,
        name: String,
        device_type: DeviceType,
        config: DeviceConfig,
        channels: Vec<Channel>,
        scan_rate: u64,
        status: String,
    ) -> Self {
        Self {
            id,
            name,
            device_type,
            config,
            channels,
            scan_rate,
            status,
        }
    }
    pub fn initialize(id: usize, name: String) -> Self {
        let mut channels = Vec::new();
        let config = DeviceConfig::Tcp(TcpConfig {
            address: "127.0.0.1".to_owned(),
            port: 502,
        });
        for i in 0..NUM_CHANNELS {
            let channel = Channel {
                id: i,
                device_id: id,
                ..Default::default()
            };
            channels.push(channel);
        }
        Device {
            id,
            name,
            device_type: DeviceType::Modbus,
            config,
            channels,
            status: "Initialized".to_owned(),
            scan_rate: 1,
        }
    }
    // To be replaced with a DOP function.
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
            id: 0,
            name: "".to_owned(),
            device_type: DeviceType::Modbus,
            config,
            channels,
            status: "Initialized".to_owned(),
            scan_rate: 1,
        }
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
