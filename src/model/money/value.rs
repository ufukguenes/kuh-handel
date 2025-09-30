use std::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug)]
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
