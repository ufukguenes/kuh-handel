use kuh_handel_lib::messages::actions::*;
use pyo3::prelude::*;

#[pymodule]
pub fn actions_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<NoAction>();
    m.add_class::<PlayerTurnDecision>();
    m.add_class::<InitialTrade>();
    m.add_class::<TradeOpponentDecision>();
    m.add_class::<SendMoney>();
    m.add_class::<Bidding>();

    Ok(())
}
