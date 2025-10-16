use serde::{Deserialize, Serialize};
use std::rc::Rc;

use crate::{
    messages::actions::Bidding,
    {
        animals::{Animal, AnimalSet},
        money::{money::Money, value::Value, wallet::Wallet},
        player::base_player::PlayerId,
    },
};

pub type Points = usize;

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
}

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
