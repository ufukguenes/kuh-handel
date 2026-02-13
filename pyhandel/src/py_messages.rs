use pyo3::prelude::*;
pub mod py_actions;
pub mod py_game_updates;
pub mod py_message_protocol;
use crate::add_submodule;

pub fn messages_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let actions = PyModule::new(m.py(), "actions")?;
    let game_updates = PyModule::new(m.py(), "game_updates")?;
    let message_protocol = PyModule::new(m.py(), "message_protocol")?;

    py_actions::actions_module_entry(&actions)?;
    py_game_updates::game_updates_module_entry(&game_updates)?;
    py_message_protocol::message_protocol_module_entry(&message_protocol)?;

    add_submodule(m, "messages".to_string(), &actions, "actions".to_string())?;
    add_submodule(
        m,
        "messages".to_string(),
        &game_updates,
        "game_updates".to_string(),
    )?;
    add_submodule(
        m,
        "messages".to_string(),
        &message_protocol,
        "message_protocol".to_string(),
    )?;
    Ok(())
}
