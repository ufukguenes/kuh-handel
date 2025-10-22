use pyo3::prelude::*;
pub mod py_base_player;
pub mod py_random_player;
pub mod py_wallet;

#[pymodule]
pub fn player_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let base_player = PyModule::new(m.py(), "base_player")?;
    let wallet = PyModule::new(m.py(), "wallet")?;

    let random_player = PyModule::new(m.py(), "random_player")?;

    py_base_player::base_player_module_entry(&base_player)?;
    py_wallet::wallet_module_entry(&wallet)?;
    py_random_player::random_player_module_entry(&random_player)?;

    m.add_submodule(&base_player);
    m.add_submodule(&wallet);
    m.add_submodule(&random_player);
    Ok(())
}
