use lib_device::{AccessType, Channel, Device, ValueType};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct WindowsOpen {
    pub plc: bool,
    pub modbus_device: bool,
    pub compressor: bool,
    pub new_device: bool,
    pub device_channels: bool,
    pub preferences: bool,
    pub channel_config: bool,
    pub device_channels_vec: [bool; 10],
    pub channel_write_value: bool,
}
#[derive(Default, Serialize, Deserialize)]
pub struct DeviceWindowsBuffer {
    pub name: String,
    pub address: String,
    pub port: String,
}
#[derive(Default, Serialize, Deserialize)]
pub struct ChannelWindowsBuffer {
    pub selected_device: Device,
    pub selected_channel: Channel,
    pub edited_channel: Channel,
    pub channel_write_value: Vec<String>,
}
