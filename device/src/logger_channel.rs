use crate::{Calculation, Channel};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LoggerChannel {
    Channel(Channel),
    Calculation(Calculation),
}
