pub enum ValueType {
    Int16,
    Real32,
    BoolType,
}

pub struct Channel {
    id: usize,
    value_type: ValueType,
    value: f32,
    index: usize,
}

impl Channel {
    pub fn new(id: usize, value_type: ValueType, index: usize) -> Self {
        let value = 0.0;
        Self {
            id,
            value_type,
            value,
            index,
        }
    }
}
