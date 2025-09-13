use std::fmt;
use std::fmt::Display;

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct Money {
    value: Value,
}

impl Money {
    pub fn new(value: Value) -> Self {
        Money { value }
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
