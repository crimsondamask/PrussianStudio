use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactLogix {
    pub name: String,
    pub id: usize,
    pub address: String,
    pub channels: Vec<LogixChannel>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub name: String,
    pub tag_value: TagValueType,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum TagValueType {
    Bool(bool),
    Int(i16),
    Dint(i32),
    Real(f32),
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct LogixChannel {
    pub id: usize,
    pub alias: String,
    pub tag: Tag,
}
