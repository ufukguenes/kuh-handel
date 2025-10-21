use crate::py_animals::Animal;
use crate::py_player::py_base_player::PlayerId;
use crate::{Money, Value};
use pyo3::prelude::*;

#[pymodule]
pub fn actions_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<NoAction>();
    m.add_class::<PlayerTurnDecision>();

    Ok(())
}

#[pyclass]
pub enum NoAction {
    Ok,
}

#[pyclass]
pub enum PlayerTurnDecision {
    Draw(),
    Trade(InitialTrade),
}

#[pyclass]
#[derive(Clone)]
pub struct InitialTrade {
    pub opponent: PlayerId,
    pub animal: Animal,
    pub animal_count: usize,
    pub amount: Vec<Money>,
}

#[pyclass]
#[derive(Clone)]
pub struct TradeOffer {
    pub challenger: PlayerId,
    pub animal: Animal,
    pub animal_count: usize,
    pub challenger_card_offer: usize,
}

#[pyclass]
#[derive(Clone)]
pub enum AuctionDecision {
    Buy,
    Sell,
}

#[pyclass]
#[derive(Clone)]
pub enum TradeOpponentDecision {
    Accept(),
    CounterOffer(Vec<Money>),
}

#[pyclass]
#[derive(Clone)]
pub enum SendMoney {
    WasBluff(),
    Amount(Vec<Money>),
}

#[pyclass]
#[derive(Clone)]
pub enum Bidding {
    Pass(),
    Bid(Value),
}
