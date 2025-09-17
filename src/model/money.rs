use std::env::var;
use std::fmt;
use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Value {
    value: u32,
}

impl Value {
    pub fn new(value: u32) -> Self {
        Value { value: value }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Money {
    value: Value,
}

impl Money {
    pub fn new(value: Value) -> Self {
        Money { value }
    }

    pub fn new_u32(value: u32) -> Self {
        Money {
            value: Value::new(value),
        }
    }

    pub fn get_value(self) -> Value {
        self.value
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
