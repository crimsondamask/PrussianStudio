use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DeviceConfig {
    Tcp(TcpConfig),
    Serial(SerialConfig),
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Parity {
    Odd,
    Even,
    NoneParity,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TcpConfig {
    pub address: String,
    pub port: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SerialConfig {
    pub com_port: String,
    pub baudrate: u32,
    pub slave: u8,
    pub parity: Parity,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        let config = TcpConfig {
            address: "127.0.0.1".to_owned(),
            port: 502,
        };
        DeviceConfig::Tcp(config)
    }
}
