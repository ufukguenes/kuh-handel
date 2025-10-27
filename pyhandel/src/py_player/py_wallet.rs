use kuh_handel_lib::player::{wallet::Affordability, wallet::Wallet};
use pyo3::prelude::*;

#[pymodule]
pub fn wallet_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<Wallet>();
    m.add_class::<Affordability>();

    Ok(())
}
