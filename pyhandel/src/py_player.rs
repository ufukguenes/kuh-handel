use pyo3::prelude::*;
pub mod py_base_player;
pub mod py_player_actions;
pub mod py_random_player;
pub mod py_simple_player;
pub mod py_wallet;

#[pymodule]
pub fn player_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let base_player = PyModule::new(m.py(), "base_player")?;
    let wallet = PyModule::new(m.py(), "wallet")?;

    let random_player = PyModule::new(m.py(), "random_player")?;
    let simple_player = PyModule::new(m.py(), "simple_player")?;
    let player_actions = PyModule::new(m.py(), "player_actions")?;

    py_base_player::base_player_module_entry(&base_player)?;
    py_wallet::wallet_module_entry(&wallet)?;
    py_random_player::random_player_module_entry(&random_player)?;
    py_simple_player::simple_player_module_entry(&simple_player)?;
    py_player_actions::player_actions_module_entry(&player_actions)?;

    m.add_submodule(&base_player);
    m.add_submodule(&wallet);
    m.add_submodule(&random_player);
    m.add_submodule(&simple_player);
    m.add_submodule(&player_actions);
    Ok(())
}
