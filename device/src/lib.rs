mod channel;
mod modbus;

use std::error::Error;

use channel::*;
use serde::{Deserialize, Serialize};
use tokio_modbus::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum DeviceType {
    Modbus,
    OpcServer,
    WebSocketServer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub address: String,
    pub port: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub device_type: DeviceType,
    pub config: DeviceConfig,
    pub channels: Vec<Channel>,
}

impl Device {
    pub fn new(
        name: String,
        device_type: DeviceType,
        config: DeviceConfig,
        channels: Vec<Channel>,
    ) -> Self {
        Self {
            name,
            device_type,
            config,
            channels,
        }
    }
    pub fn fetch_data_tcp(&mut self) -> Result<(), Box<dyn Error>> {
        let ip = &self.config.address;
        let port = &self.config.port;
        let socket = format!("{}:{}", ip, port).parse()?;

        let mut ctx = sync::tcp::connect(socket)?;
        for channel in &mut self.channels {
            channel.read_value(&mut ctx);
        }

        Ok(())
    }
}

impl Default for Device {
    fn default() -> Self {
        Device {
            name: "".to_owned(),
            device_type: DeviceType::Modbus,
            config: DeviceConfig {
                address: "192.168.0.1".to_owned(),
                port: 502,
            },
            channels: Vec::new(),
        }
    }
}
