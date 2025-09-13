use std::fmt;

struct Animal {
    value: u32,
    occurences: u32,
    inflation: [u32; occurences]
}

impl Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}