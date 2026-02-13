use pyo3::prelude::*;
pub mod py_animals;
pub mod py_client;
pub mod py_messages;
pub mod py_player;

pub type Money = usize;
pub type Value = usize;
pub type Points = usize;

#[pymodule]
fn pyhandel(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let messages = PyModule::new(m.py(), "messages")?;
    py_messages::messages_module_entry(&messages)?;

    let player = PyModule::new(m.py(), "player")?;
    py_player::player_module_entry(&player)?;

    let animals = PyModule::new(m.py(), "animals")?;
    py_animals::animal_module_entry(&animals)?;

    let client = PyModule::new(m.py(), "client")?;
    py_client::client_module_entry(&client)?;

    add_submodule(m, "pyhandel".to_string(), &messages, "messages".to_string())?;
    add_submodule(m, "pyhandel".to_string(), &player, "player".to_string())?;
    add_submodule(m, "pyhandel".to_string(), &animals, "animals".to_string())?;
    add_submodule(m, "pyhandel".to_string(), &client, "client".to_string())?;

    Ok(())
}

fn add_submodule(
    parent_module: &Bound<'_, PyModule>,
    parent_name: String,
    child_module: &Bound<'_, PyModule>,
    child_name: String,
) -> PyResult<()> {
    parent_module.add_submodule(&child_module)?;
    parent_module
        .py()
        .import("sys")?
        .getattr("modules")?
        .set_item(format!("{parent_name}.{child_name}"), child_module)?;

    Ok(())
}
