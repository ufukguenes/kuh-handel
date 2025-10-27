use serde::{Deserialize, Serialize};

use crate::Value;
use pyo3::prelude::*;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

#[pyclass(unsendable)]
#[derive(Debug)]
pub enum AnimalError {
    InvalidArgument,
    InvalidState,
    InvalidDraw,
}

#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimalSet {
    animal: Animal,
    inflation: Vec<Value>,
    draw_count: RefCell<usize>,
    animals: Vec<Rc<Animal>>,
}

impl Display for AnimalSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Animal {}", self.animal)
    }
}

impl AnimalSet {
    pub fn new(value: usize, inflation_numbers: Vec<usize>) -> AnimalSet {
        let animal = Animal::new(value);
        let animals = inflation_numbers
            .clone()
            .iter()
            .map(|_| Rc::new(animal.clone()))
            .collect();

        AnimalSet {
            animal: animal,
            inflation: inflation_numbers,
            draw_count: RefCell::new(0),
            animals: animals,
        }
    }

    pub fn get_next_inflation(&self) -> Value {
        self.inflation[*self.draw_count.borrow()]
    }

    pub fn occurrences(&self) -> usize {
        self.inflation.len()
    }

    pub fn increase_draw_count(&self) {
        *self.draw_count.borrow_mut() += 1;
    }

    pub fn animal(&self) -> &Animal {
        &self.animal
    }

    pub fn animals(&self) -> &Vec<Rc<Animal>> {
        &self.animals
    }
}

#[pyclass(unsendable)]
#[derive(Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize, Debug, PartialOrd, Ord)]
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
