use super::money::{Money, Value};
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum AnimalError {
    InvalidArgument,
    InvalidState,
}

type AnimalResult<T> = Result<T, AnimalError>;

pub struct DefaultAnimalFactory {}

impl AnimalFactory for DefaultAnimalFactory {
    fn new(value_number: u32, inflation: Vec<u32>) -> AnimalSet {
        let value = Value::new(value_number);
        let money_value: Money = Money::new(value);
        AnimalSet {
            animal: Animal::new(money_value),
            inflation: inflation,
        }
    }
}

pub trait AnimalFactory {
    fn new(value: u32, inflation: Vec<u32>) -> AnimalSet;
}

pub struct AnimalSet {
    animal: Animal,
    inflation: Vec<u32>,
}

impl Display for AnimalSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Animal {}", self.animal)
    }
}

impl AnimalSet {
    fn occurrences(&self) -> usize {
        self.inflation.len()
    }

    fn animals(&self) -> Vec<Animal> {
        vec![self.animal; self.occurrences()]
    }
}

#[derive(Clone, Copy)]
pub struct Animal {
    value: Money,
}

impl Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Animal {
    pub fn new(value: Money) -> Self {
        Animal { value }
    }
}
