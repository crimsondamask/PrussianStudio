// use std::error::Error;

use std::fmt::{write, Display};

use serde::{Deserialize, Serialize};
use tokio_modbus::prelude::{sync::Context, *};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum ValueType {
    Int16,
    Real32,
    BoolType,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum AccessType {
    Read,
    Write,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Channel {
    pub id: usize,
    pub tag: String,
    pub value_type: ValueType,
    pub access_type: AccessType,
    pub value: f32,
    pub index: u16,
    pub status: String,
}

impl Channel {
    pub fn new(
        id: usize,
        value_type: ValueType,
        access_type: AccessType,
        index: u16,
        tag: String,
        status: String,
    ) -> Self {
        let value = 0.0;
        Self {
            id,
            tag,
            value_type,
            access_type,
            value,
            index,
            status,
        }
    }
    pub fn read_value(&mut self, ctx: &mut Context) {
        match self.value_type {
            ValueType::Int16 => {
                if let Ok(value) = ctx.read_holding_registers(self.index, 1) {
                    self.value = value[0] as f32;
                }
            }
            ValueType::Real32 => {
                if let Ok(data) = ctx.read_holding_registers(self.index, 2) {
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
    pub fn write_value(&mut self, ctx: &mut Context) {
        match self.value_type {
            ValueType::Int16 => {
                let value = self.value as u16;

                match ctx.write_single_register(self.index, value) {
                    Ok(_) => {
                        self.status = "Value written successfully!".to_owned();
                    }
                    Err(e) => {
                        self.status = format!("ERROR!: {}", e);
                    }
                }
            }
            ValueType::Real32 => {
                let value = self.value as u16;

                match ctx.write_single_register(self.index, value) {
                    Ok(_) => {
                        self.status = "Value written successfully!".to_owned();
                    }
                    Err(e) => {
                        self.status = format!("ERROR!: {}", e);
                    }
                }
            }
            ValueType::BoolType => {
                let value = self.value as u16;
                match value {
                    1 => {
                        if let Ok(_) = ctx.write_single_coil(self.index, true) {
                            self.status = "Value written successfully!".to_owned();
                        } else {
                            self.status = "Couldn't write value".to_owned();
                        }
                    }
                    0 => {
                        if let Ok(_) = ctx.write_single_coil(self.index, false) {
                            self.status = "Value written successfully!".to_owned();
                        } else {
                            self.status = "Couldn't write value".to_owned();
                        }
                    }
                    _ => self.status = "Only bit values are allowed!.".to_owned(),
                }
            }
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            id: 0,
            tag: "".to_owned(),
            value_type: ValueType::Int16,
            access_type: AccessType::Read,
            value: 0.0,
            index: 0,
            status: "Initialized".to_owned(),
        }
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CH{}", self.id)
    }
}
impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value_type = match self {
            ValueType::Int16 => "Int",
            ValueType::Real32 => "Real",
            ValueType::BoolType => "Bool",
        };
        write!(f, "{}", value_type)
    }
}

impl Display for AccessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let access_type = match self {
            AccessType::Read => "Read",
            AccessType::Write => "Write",
        };
        write!(f, "{}", access_type)
    }
}

impl Default for ValueType {
    fn default() -> Self {
        ValueType::Int16
    }
}
impl Default for AccessType {
    fn default() -> Self {
        AccessType::Read
    }
}
