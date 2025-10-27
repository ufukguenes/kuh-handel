use crate::Value;
use kuh_handel_lib::animals::{Animal, AnimalError, AnimalSet};

use pyo3::prelude::*;

#[pymodule]
pub fn animal_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<AnimalError>();
    m.add_class::<Animal>();
    m.add_class::<AnimalSet>();

    Ok(())
}
