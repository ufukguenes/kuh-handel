use std::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::model::money::value::Value;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Debug)]
pub struct Money {
    pub value: Value,
}

impl Money {
    pub fn new(value: Value) -> Self {
        Money { value }
    }

    pub fn new_usize(value: usize) -> Self {
        Money {
            value: Value::new(value),
        }
    }

    pub fn value(self) -> Value {
        self.value
    }

    pub fn as_usize(self) -> usize {
        self.value.value()
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
