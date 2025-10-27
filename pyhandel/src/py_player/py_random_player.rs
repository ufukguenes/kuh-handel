use kuh_handel_lib::player::random_player::RandomPlayerActions as CorePlayer;
use pyo3::prelude::*;

#[pymodule]
pub fn random_player_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<RandomPlayerActions>();

    Ok(())
}

#[pyclass(unsendable)]
pub struct RandomPlayerActions {
    pub inner: Option<CorePlayer>,
}

#[pymethods]
impl RandomPlayerActions {
    #[new]
    pub fn new(id: String, seed: u64) -> Self {
        RandomPlayerActions {
            inner: Some(CorePlayer::new(id, seed)),
        }
    }
}
