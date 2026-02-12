use kuh_handel_lib::player::base_player::Player;
use pyo3::prelude::*;

pub fn base_player_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<Player>();

    Ok(())
}
