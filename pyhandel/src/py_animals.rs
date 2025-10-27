use crate::Value;
use kuh_handel_lib::animals::Animal as CoreAnimal;
use kuh_handel_lib::animals::AnimalSet as CoreAnimalSet;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pymodule]
pub fn animal_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<AnimalError>();
    m.add_class::<Animal>();
    m.add_class::<AnimalSet>();

    Ok(())
}

#[pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimalSet {
    pub animal: Animal,
    pub inflation: Vec<Value>,
    pub animals: Vec<Animal>,
    pub draw_count: usize,
}

#[pymethods]
impl AnimalSet {
    #[new]
    pub fn new(value: usize, inflation_numbers: Vec<usize>) -> AnimalSet {
        let animal = Animal::new(value);
        let num_animal = inflation_numbers.len();
        AnimalSet {
            animal: Animal::new(value),
            inflation: inflation_numbers,
            animals: vec![animal; num_animal],
            draw_count: 0,
        }
    }
}

impl AnimalSet {
    pub fn convert_to_rs(&self) -> CoreAnimalSet {
        CoreAnimalSet::new(self.animal.value(), self.inflation.clone())
    }
}

#[pyclass]
#[derive(Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize, Debug, PartialOrd, Ord)]
pub struct Animal {
    inner: CoreAnimal,
}

#[pymethods]
impl Animal {
    #[new]
    pub fn new(value: Value) -> Self {
        Animal {
            inner: CoreAnimal::new(value),
        }
    }

    pub fn value(&self) -> Value {
        self.inner.value()
    }
}

impl Animal {
    pub fn convert_to_rs(&self) -> &CoreAnimal {
        &self.inner
    }

    pub fn convert_to_py(animal: CoreAnimal) -> Self {
        Animal { inner: animal }
    }
}

#[pyclass]
pub enum AnimalError {
    InvalidArgument,
    InvalidState,
    InvalidDraw,
}
