use pyo3::prelude::*;
pub mod py_animals;
pub mod py_client;
pub mod py_messages;
pub mod py_player;

pub type Money = usize;
pub type Value = usize;
pub type Points = usize;
pub type PlayerId = String;

#[pymodule]
fn animals(m: &Bound<'_, PyModule>) -> PyResult<()> {
    py_animals::animal_module_entry(m)?;
    Ok(())
}

#[pymodule]
fn messages(m: &Bound<'_, PyModule>) -> PyResult<()> {
    py_messages::messages_module_entry(m)?;
    Ok(())
}

#[pymodule]
fn player(m: &Bound<'_, PyModule>) -> PyResult<()> {
    py_player::player_module_entry(m)?;
    Ok(())
}

#[pymodule]
fn client(m: &Bound<'_, PyModule>) -> PyResult<()> {
    py_client::client_module_entry(m)?;
    Ok(())
}
