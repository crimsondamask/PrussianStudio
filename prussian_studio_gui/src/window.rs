use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct WindowsOpen {
    pub device: bool,
    pub new_device: bool,
    pub device_channels: bool,
}
#[derive(Default, Serialize, Deserialize)]
pub struct DeviceWindowsBuffer {
    pub name: String,
    pub address: String,
    pub port: String,
}
