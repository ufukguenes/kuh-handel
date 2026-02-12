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
fn pyhandel(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let messages = PyModule::new(m.py(), "messages")?;
    py_messages::messages_module_entry(&messages)?;

    let player = PyModule::new(m.py(), "player")?;
    py_player::player_module_entry(&player)?;

    let animal = PyModule::new(m.py(), "animal")?;
    py_animals::animal_module_entry(&animal)?;

    let client = PyModule::new(m.py(), "client")?;
    py_client::client_module_entry(&client)?;

    m.add_submodule(&messages)?;
    m.add_submodule(&player)?;
    m.add_submodule(&animal)?;
    m.add_submodule(&client)?;

    Ok(())
}
