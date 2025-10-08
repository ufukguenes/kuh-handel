use serde::{Deserialize, Serialize};

use super::money::value::Value;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug)]
pub enum AnimalError {
    InvalidArgument,
    InvalidState,
    InvalidDraw,
}

type AnimalResult<T> = Result<T, AnimalError>;

pub struct DefaultAnimalSetFactory {}

impl AnimalSetFactory for DefaultAnimalSetFactory {
    fn new(value_number: usize, inflation_numbers: Vec<usize>) -> AnimalSet {
        let inflation: Vec<Value> = inflation_numbers.iter().map(|e| Value::new(*e)).collect();
        let value = Value::new(value_number);

        let animal = Animal::new(value);
        let animals = inflation
            .clone()
            .iter()
            .map(|_| Rc::new(animal.clone()))
            .collect();

        AnimalSet {
            animal: animal,
            inflation: inflation,
            draw_count: 0,
            animals: animals,
        }
    }

    fn new_from_value(value_number: usize, inflation: Vec<Value>) -> AnimalSet {
        let value = Value::new(value_number);

        let animal = Animal::new(value);
        let animals = inflation
            .clone()
            .iter()
            .map(|_| Rc::new(animal.clone()))
            .collect();

        AnimalSet {
            animal: animal,
            inflation: inflation,
            draw_count: 0,
            animals: animals,
        }
    }
}

pub trait AnimalSetFactory {
    fn new(value: usize, inflation: Vec<usize>) -> AnimalSet;
    fn new_from_value(value: usize, inflation: Vec<Value>) -> AnimalSet;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimalSet {
    animal: Animal,
    inflation: Vec<Value>,
    draw_count: usize,
    animals: Vec<Rc<Animal>>,
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

    pub fn animal(&self) -> &Animal {
        &self.animal
    }

    pub fn animals(&self) -> &Vec<Rc<Animal>> {
        &self.animals
    }

    fn draw_animal(&mut self) -> Result<Value, AnimalError> {
        if self.draw_count == self.occurrences() {
            return Err(AnimalError::InvalidDraw);
        }
        self.draw_count += 1;
        return Ok(self.inflation[self.draw_count - 1]);
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
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
