use kuh_handel_lib::messages::message_protocol::*;
use pyo3::prelude::*;

#[pymodule]
pub fn message_protocol_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<ActionMessage>();
    m.add_class::<StateMessage>();

    Ok(())
}
