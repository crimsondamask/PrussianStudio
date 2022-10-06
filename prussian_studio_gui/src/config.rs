use lib_device::Device;
use lib_logger::Logger;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub devices: Vec<Device>,
    pub loggers: Vec<Logger>,
}
