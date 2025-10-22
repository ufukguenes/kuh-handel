use crate::py_messages::py_actions::InitialTrade;
use kuh_handel_lib::messages::actions::InitialTrade as CoreInitialTrade;
use kuh_handel_lib::player::random_player::RandomPlayerActions as CorePlayer;
use pyo3::{exceptions::PyRuntimeError, prelude::*, PyErrArguments};

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
