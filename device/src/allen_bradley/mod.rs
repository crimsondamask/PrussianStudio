use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactLogix {
    name: String,
    address: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    name: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct LogixChannel {
    id: usize,
    tag: String,
}
