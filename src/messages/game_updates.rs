use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{
    messages::actions::Bidding,
    model::{
        animals::{Animal, AnimalSet},
        money::{value::Value, wallet::Wallet},
        player::base_player::PlayerId,
    },
};

type Points = usize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuctionRound {
    pub host: PlayerId,
    pub animal: Rc<Animal>,
    pub bids: Vec<(PlayerId, Bidding)>,
}

/// After each game event, all players are informed about what happened.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameUpdate {
    /// The action update is sent after an auction has finished.
    Auction {
        rounds: AuctionRound,
        transfer: MoneyTransfer,
    },
    Trade {
        challenger: PlayerId,
        opponent: PlayerId,
        animal: Animal,
        animal_count: AnimalTradeCount,
        challenger_card_offer: usize,
        opponent_card_offer: Option<usize>,
        receiver: PlayerId,
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MoneyTransfer {
    pub from: PlayerId,
    pub to: PlayerId,
    pub card_amount: usize,
    pub min_value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[repr(usize)]
pub enum AnimalTradeCount {
    One = 1,
    Two = 2,
}
