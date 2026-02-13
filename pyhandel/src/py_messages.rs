use pyo3::prelude::*;
pub mod py_actions;
pub mod py_game_updates;
pub mod py_message_protocol;

pub fn messages_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let actions = PyModule::new(m.py(), "actions")?;
    let game_updates = PyModule::new(m.py(), "game_updates")?;
    let message_protocol = PyModule::new(m.py(), "message_protocol")?;

    py_actions::actions_module_entry(&actions)?;
    py_game_updates::game_updates_module_entry(&game_updates)?;
    py_message_protocol::message_protocol_module_entry(&message_protocol)?;

    m.add_submodule(&actions);
    m.add_submodule(&game_updates);
    m.add_submodule(&message_protocol);
    Ok(())
}
