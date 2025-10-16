use std::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug)]
pub struct Value {
    value: usize,
}

impl Value {
    pub fn new(value: usize) -> Self {
        Value { value: value }
    }

    pub fn value(&self) -> usize {
        self.value
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
