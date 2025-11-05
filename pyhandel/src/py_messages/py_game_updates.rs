use kuh_handel_lib::messages::game_updates::*;
use pyo3::prelude::*;

#[pymodule]
pub fn game_updates_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<AuctionRound>();
    m.add_class::<GameUpdate>();
    m.add_class::<AuctionKind>();
    m.add_class::<MoneyTransfer>();
    m.add_class::<MoneyTrade>();
    m.add_class::<TradeOffer>();

    Ok(())
}
