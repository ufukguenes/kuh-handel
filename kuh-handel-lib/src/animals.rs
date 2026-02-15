use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::Value;
use pyo3::prelude::*;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::sync::Arc;

#[pyclass()]
#[derive(Debug)]
pub enum AnimalError {
    InvalidArgument,
    InvalidState,
    InvalidDraw,
}

#[pyclass()]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimalSet {
    #[pyo3(get, set)]
    animal: Animal,
    #[pyo3(get, set)]
    inflation: Vec<Value>,
    draw_count: usize,
    animals: Vec<Arc<Animal>>,
}

impl Display for AnimalSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Animal {}", self.animal)
    }
}

#[pymethods]
impl AnimalSet {
    #[new]
    pub fn new(value: usize, inflation_numbers: Vec<usize>) -> AnimalSet {
        let animal = Animal::new(value);
        let animals = inflation_numbers
            .clone()
            .iter()
            .map(|_| Arc::new(animal.clone()))
            .collect();

        AnimalSet {
            animal: animal,
            inflation: inflation_numbers,
            draw_count: 0,
            animals: animals,
        }
    }
}

impl AnimalSet {
    pub fn get_next_inflation(&self) -> Value {
        self.inflation[self.draw_count]
    }

    pub fn occurrences(&self) -> usize {
        self.inflation.len()
    }

    pub fn increase_draw_count(&mut self) {
        self.draw_count += 1;
    }

    pub fn animal(&self) -> &Animal {
        &self.animal
    }

    pub fn animals(&self) -> &Vec<Arc<Animal>> {
        &self.animals
    }
}

#[pyclass()]
#[derive(Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize, Debug, PartialOrd, Ord)]
pub struct Animal {
    #[pyo3(get, set)]
    value: Value,
}

impl Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[pymethods]
impl Animal {
    #[new]
    pub fn new(value: Value) -> Self {
        Animal { value }
    }
}

impl Animal {
    pub fn value(&self) -> Value {
        self.value
    }
}
