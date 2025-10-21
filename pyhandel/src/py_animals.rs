use crate::Value;
use kuh_handel_lib::animals::Animal as CoreAnimal;
use pyo3::prelude::*;

#[pymodule]
pub fn animal_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<AnimalError>();

    Ok(())
}

#[pyclass]
#[derive(Clone)]
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

#[pyclass]
pub enum AnimalError {
    InvalidArgument,
    InvalidState,
    InvalidDraw,
}
