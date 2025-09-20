use std::env::var;
use std::fmt;
use std::fmt::Display;

use crate::model::money::value::Value;

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
