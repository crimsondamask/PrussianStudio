mod channel;
mod modbus;

use channel::*;

pub enum DeviceType {
    Modbus,
    OpcServer,
    WebSocketServer,
}

pub struct DeviceConfig {
    pub address: String,
    pub port: usize,
}

pub struct Device {
    name: String,
    device_type: DeviceType,
    config: DeviceConfig,
    channels: Vec<Channel>,
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
    pub fn fetch_data() {
        todo!()
    }
}
