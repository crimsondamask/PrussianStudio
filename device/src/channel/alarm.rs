use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum AlarmType {
    High,
    // HHigh,
    Low,
    // LLow,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Alarm {
    pub alarm_type: AlarmType,
    pub active: bool,
    pub enabled: bool,
    pub setpoint: f32,
}

impl Alarm {
    pub fn process_alarm(&mut self, value: f32) {
        match self.alarm_type {
            AlarmType::Low => {
                if value < self.setpoint {
                    self.active = true;
                } else {
                    self.active = false;
                }
            }
            AlarmType::High => {
                if value > self.setpoint {
                    self.active = true;
                } else {
                    self.active = false;
                }
            }
        }
    }
}

impl Display for AlarmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let alarm_type = match self {
            AlarmType::Low => "Low",
            AlarmType::High => "High",
            // AlarmType::LLow => "LLow",
            // AlarmType::HHigh => "HHigh",
        };
        write!(f, "{}", alarm_type)
    }
}

impl Default for Alarm {
    fn default() -> Self {
        Self {
            alarm_type: AlarmType::High,
            active: false,
            enabled: false,
            setpoint: 0.0,
        }
    }
}
