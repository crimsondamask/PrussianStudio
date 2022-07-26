use std::path::PathBuf;

use lib_device::{Channel, Device, DeviceConfig};
use lib_logger::{ChannelPattern, LoggerType};
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
    pub logger_configure: bool,
    pub save_config: bool,
    pub load_config: bool,
    pub confirm_exit: bool,
}
#[derive(Default, Serialize, Deserialize)]
pub enum DeviceType {
    #[default]
    Tcp,
    Serial,
}
#[derive(Default, Serialize, Deserialize)]
pub struct DeviceWindowsBuffer {
    pub device_type: DeviceType,
    pub name: String,
    pub address: String,
    pub path: String,
    pub port: String,
    pub baudrate: String,
    pub slave: String,
    pub config: DeviceConfig,
    pub status: String,
    pub scan_rate: u64,
}
#[derive(Default, Serialize, Deserialize)]
pub struct ChannelWindowsBuffer {
    pub selected_device: Device,
    pub device_id: usize,
    pub selected_channel: Channel,
    pub edited_channel: Channel,
    pub channel_write_value: Vec<String>,
}
#[derive(Default, Serialize, Deserialize)]
pub struct LoggerWindowBuffer {
    pub logger_name: String,
    pub logger_type: LoggerType,
    pub log_rate: usize,
    pub path: PathBuf,
    pub channel_pattern: ChannelPattern,
    pub pattern_str: String,
    pub is_logging: bool,
}
