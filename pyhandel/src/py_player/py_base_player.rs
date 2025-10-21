use pyo3::prelude::*;

#[pymodule]
pub fn base_player_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<PlayerId>();

    Ok(())
}

#[pyclass]
#[derive(Clone)]
pub struct PlayerId {
    pub name: String,
}

#[pymethods]
impl PlayerId {
    #[new]
    pub fn new(name: String) -> Self {
        PlayerId { name: name }
    }
}
