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

const DEVICE_NUM_CHANNELS: usize = 20;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct DataBlock {
    pub holding_regs: Vec<i16>,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct JsonWriteChannel {
    pub device_id: usize,
    pub channel: usize,
    pub value: f32,
}

#[derive(Clone)]
pub enum DeviceMsg {
    Reconnect(DeviceConfig),
    WriteChannel(JsonWriteChannel),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Modbus,
    OpcServer,
    WebSocketServer,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Device {
    pub id: usize,
    pub name: String,
    pub device_type: DeviceType,
    pub config: DeviceConfig,
    pub channels: Vec<Channel>,
    pub data_block: DataBlock,
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
        data_block: DataBlock,
        scan_rate: u64,
        status: String,
    ) -> Self {
        Self {
            id,
            name,
            device_type,
            config,
            channels,
            data_block,
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
        let data_block = DataBlock {
            holding_regs: Vec::new(),
        };
        for i in 0..DEVICE_NUM_CHANNELS {
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
            data_block,
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

        let data_block = DataBlock {
            holding_regs: Vec::new(),
        };
        for i in 0..DEVICE_NUM_CHANNELS {
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
            data_block,
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

pub fn get_register_list(device: &Device) -> Vec<u16> {
    let mut register_list: Vec<u16> = Vec::new();

    let _: Vec<_> = device
        .channels
        .iter()
        .filter(|channel| channel.enabled)
        .map(|channel| match channel.value_type {
            ValueType::Int16 => register_list.push(channel.index),
            ValueType::Real32 => {
                register_list.push(channel.index);
                register_list.push(channel.index + 1);
            }
            _ => {}
        })
        .collect();

    let start = register_list.iter().min();
    let end = register_list.iter().max();

    let mut register_vec = Vec::new();
    if let (Some(min), Some(max)) = (start, end) {
        register_vec = (*min..=*max).collect();
    }
    register_vec
}

pub fn channel_values_from_buffer(
    mut device: Device,
    register_list: Vec<u16>,
    data_buffer: Vec<u16>,
) -> Device {
    // We get the data as a u16 Vec and we assign each value to its corresponding channel.
    if !data_buffer.is_empty() {
        let mut channels_to_send = Vec::new();
        for channel in device.channels {
            let mut edited_channel = channel.clone();
            for (i, register) in register_list.iter().enumerate() {
                if edited_channel.enabled && edited_channel.index == *register {
                    match edited_channel.value_type {
                        ValueType::Int16 => {
                            edited_channel.value = data_buffer[i] as f32;
                        }
                        ValueType::Real32 => {
                            let data_32bit_rep =
                                ((data_buffer[i] as u32) << 16) | data_buffer[i + 1] as u32;
                            let data_32_array = data_32bit_rep.to_ne_bytes();
                            edited_channel.value = f32::from_ne_bytes(data_32_array);
                        }
                        _ => {}
                    }
                }
            }
            channels_to_send.push(edited_channel);
        }
        device.channels = channels_to_send;
    }
    device
}
