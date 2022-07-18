// use std::error::Error;

use serde::{Deserialize, Serialize};
use tokio_modbus::prelude::{sync::Context, *};

#[derive(Debug, Serialize, Deserialize)]
pub enum ValueType {
    Int16,
    Real32,
    BoolType,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum AccessType {
    Read,
    Write,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    pub id: usize,
    pub value_type: ValueType,
    pub access_type: AccessType,
    pub value: f32,
    pub index: u16,
}

impl Channel {
    pub fn new(id: usize, value_type: ValueType, access_type: AccessType, index: u16) -> Self {
        let value = 0.0;
        Self {
            id,
            value_type,
            access_type,
            value,
            index,
        }
    }
    pub fn read_value(&mut self, ctx: &mut Context) {
        match self.value_type {
            ValueType::Int16 => {
                if let Ok(value) = ctx.read_input_registers(self.index, 1) {
                    self.value = value[0] as f32;
                }
            }
            ValueType::Real32 => {
                if let Ok(data) = ctx.read_input_registers(self.index, 2) {
                    let data_32bit_rep = ((data[0] as u32) << 16) | data[1] as u32;
                    let data_32_array = data_32bit_rep.to_ne_bytes();
                    self.value = f32::from_ne_bytes(data_32_array);
                }
            }
            ValueType::BoolType => {
                if let Ok(states) = ctx.read_coils(self.index, 1) {
                    self.value = match states[0] {
                        true => 0.0,
                        false => 1.0,
                    };
                }
            }
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            id: 0,
            value_type: ValueType::BoolType,
            access_type: AccessType::Read,
            value: 0.0,
            index: 0,
        }
    }
}
