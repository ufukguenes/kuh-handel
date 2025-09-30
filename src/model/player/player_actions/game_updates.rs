use serde::{Deserialize, Serialize};

use crate::model::{
    animals::{Animal, AnimalSet},
    money::{money::Money, value::Value, wallet::Wallet},
    player::base_player::PlayerId,
};

type Points = usize;

pub struct AuctionRound {
    host: PlayerId,
    animal: Animal,
    bids: Vec<(PlayerId, Bidding)>,
}

/// After each game event, all players are informed about what happened.
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

pub struct MoneyTransfer {
    from: PlayerId,
    to: PlayerId,
    card_amount: usize,
    min_value: Value,
}

pub enum Bidding {
    Pass,
    Bid(Money),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[repr(usize)]
pub enum AnimalTradeCount {
    One = 1,
    Two = 2,
}
