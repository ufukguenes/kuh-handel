use kuh_handel_lib::animals::Animal;
use pyo3::prelude::*;

#[pyclass]
struct K {
    a: Animal,
}
