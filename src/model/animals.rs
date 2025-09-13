use std::fmt;
use std::fmt::Display;

pub struct Animal {
    value: u32,
    occurrences: u32,
    inflation: [u32; 4]
}

impl Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Animal {
    pub fn new(value: u32, occurrences: u32, inflation: [u32; 4]) -> Self {
        Animal { value, occurrences, inflation }
    }
}