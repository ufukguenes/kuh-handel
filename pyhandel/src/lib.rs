use pyo3::{
    prelude::*,
    types::{PyInt, PyList},
};
pub mod py_animals;
pub mod py_client;
pub mod py_messages;
pub mod py_player;

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

    let py_int = m.py().get_type::<PyInt>();
    m.add("Money", &py_int)?;
    m.add("Value", &py_int)?;
    m.add("Points", &py_int)?;
    Ok(())
}

fn add_submodule(
    parent_module: &Bound<'_, PyModule>,
    parent_path: String,
    child_module: &Bound<'_, PyModule>,
    child_name: String,
) -> PyResult<()> {
    let name = format!("{parent_path}.{child_name}");
    parent_module.add_submodule(&child_module)?;
    parent_module
        .py()
        .import("sys")?
        .getattr("modules")?
        .set_item(name.clone(), child_module)?;

    child_module.setattr("__name__", name)?;
    child_module.setattr("__path__", PyList::empty(parent_module.py()))?;

    Ok(())
}
