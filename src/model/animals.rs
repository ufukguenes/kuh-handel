use super::money::{Money, Value};
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum AnimalError {
    InvalidArgument,
    InvalidState,
    InvalidDraw,
}

type AnimalResult<T> = Result<T, AnimalError>;

pub struct DefaultAnimalSetFactory {}

impl AnimalSetFactory for DefaultAnimalSetFactory {
    fn new(value_number: u32, inflation_numbers: Vec<u32>) -> AnimalSet {
        let inflation = inflation_numbers.iter().map(|e| Value::new(*e)).collect();
        let value = Value::new(value_number);
        AnimalSet {
            animal: Animal::new(value),
            inflation: inflation,
            draw_count: 0,
        }
    }
}

pub trait AnimalSetFactory {
    fn new(value: u32, inflation: Vec<u32>) -> AnimalSet;
}

pub struct AnimalSet {
    animal: Animal,
    inflation: Vec<Value>,
    draw_count: usize,
}

impl Display for AnimalSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Animal {}", self.animal)
    }
}

impl AnimalSet {
    pub fn occurrences(&self) -> usize {
        self.inflation.len()
    }

    pub fn animals(&self) -> Vec<Animal> {
        vec![self.animal; self.occurrences()]
    }

    fn draw_animal(&mut self) -> Result<Value, AnimalError> {
        if self.draw_count == self.occurrences() {
            return Err(AnimalError::InvalidDraw);
        }
        self.draw_count += 1;
        return Ok(self.inflation[self.draw_count - 1]);
    }
}

#[derive(Clone, Copy)]
pub struct Animal {
    value: Value,
}

impl Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Animal {
    pub fn new(value: Value) -> Self {
        Animal { value }
    }

    pub fn value(&self) -> Value {
        self.value
    }
}
