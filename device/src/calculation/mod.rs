use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Calculation {
    pub id: usize,
    pub value: f32,
    pub tag: String,
}
