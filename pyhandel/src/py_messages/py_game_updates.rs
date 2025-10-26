use crate::py_animals::{Animal, AnimalSet};
use crate::py_messages::py_actions::Bidding;
use crate::py_player::py_wallet::Wallet;
use crate::PlayerId;
use crate::{Money, Points, Value};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pymodule]
pub fn game_updates_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<AuctionRound>();
    m.add_class::<GameUpdate>();
    m.add_class::<AuctionKind>();
    m.add_class::<MoneyTransfer>();
    m.add_class::<MoneyTrade>();

    Ok(())
}

#[pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuctionRound {
    pub host: PlayerId,
    pub animal: Animal,
    pub bids: Vec<(PlayerId, Bidding)>,
}

#[pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameUpdate {
    /// The action update is sent after an auction has finished.
    Auction(AuctionKind),
    Trade {
        challenger: PlayerId,
        opponent: PlayerId,
        animal: Animal,
        animal_count: usize,
        receiver: PlayerId,
        money_trade: MoneyTrade,
    },
    Start {
        wallet: Wallet,
        players_in_turn_order: Vec<PlayerId>,
        animals: Vec<AnimalSet>,
    },
    End {
        ranking: Vec<(PlayerId, Points)>,
    },
    ExposePlayer {
        player: PlayerId,
        wallet: Wallet,
    },
    Inflation(Money),
}

#[pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuctionKind {
    NoBiddings {
        host_id: PlayerId,
        animal: Animal,
    },
    NormalAuction {
        rounds: AuctionRound,
        from: PlayerId,
        to: PlayerId,
        money_transfer: MoneyTransfer,
    },
}

#[pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MoneyTransfer {
    Public {
        card_amount: usize,
        min_value: Value,
    },
    Private {
        amount: Vec<Money>,
    },
}

#[pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MoneyTrade {
    Public {
        challenger_card_offer: usize,
        opponent_card_offer: Option<usize>,
    },
    Private {
        challenger_card_offer: Vec<Money>,
        opponent_card_offer: Option<Vec<Money>>,
    },
}
