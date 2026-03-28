use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::player::wallet::Wallet;
use crate::{Money, Value};
use crate::{
    messages::actions::Bidding,
    {
        animals::{Animal, AnimalSet},
        player::base_player::PlayerId,
    },
};
use pyo3::prelude::*;

/// The points in the game.
pub type Points = usize;

/// Information about the currently running auction
#[pyclass()]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuctionRound {
    /// the player that hosts the current auction
    #[pyo3(get)]
    pub host: PlayerId,

    /// the animal that is auctioned off by the host
    pub animal: Arc<Animal>,

    /// the current bids that have been placed until now by all other players
    #[pyo3(get)]
    pub bids: Vec<(PlayerId, Bidding)>,
}

#[pymethods]
impl AuctionRound {
    #[getter]
    pub fn animal(&self) -> Animal {
        *self.animal.clone()
    }
}

/// Information about a trade offer from another player
#[pyclass()]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradeOffer {
    /// id of the player that initiated the trade
    #[pyo3(get)]
    pub challenger: PlayerId,
    /// The animal that is offered for the trade
    #[pyo3(get)]
    pub animal: Animal,
    /// the number of animals that are going to be traded
    #[pyo3(get)]
    pub animal_count: usize,
    /// the number of cards/ bills the challenger has offered, the actual card values are hidden
    #[pyo3(get)]
    pub challenger_card_offer: usize,
}

/// After each game event, all players are informed about what happened.
#[pyclass()]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameUpdate {
    /// Sent after an auction has finished.
    Auction(AuctionKind),

    /// Sent after a trade is finished
    Trade {
        /// the player id who initiated the trade
        challenger: PlayerId,
        /// the player id who was chosen for the trade by the challenger
        opponent: PlayerId,
        /// the animal that was traded
        animal: Animal,
        /// the number of animals that have been traded
        animal_count: usize,
        /// the player id of the player who has placed the larger amount of money or if the same amount was placed, the challenger
        receiver: PlayerId,
        /// if the player this message is sent to is either the challenger or opponent it receives a Private MoneyTrade with more information about the trade, else the player receives a Public MoneyTrade
        money_trade: MoneyTrade,
    },

    /// Sent after a game started
    Start {
        /// the wallet each player is handed out at the beginning of the game, with the initial amount of money
        wallet: Wallet,
        /// the players sorted by their id in the order they are going to play
        players_in_turn_order: Vec<PlayerId>,
        /// the animals that exist in this game
        animals: Vec<AnimalSet>,
    },

    /// Sent after the game is finished
    End {
        /// the points each player achieved in this game
        ranking: Vec<(PlayerId, Points)>,
        /// overview of actions that have been removed automatically
        illegal_moves_made: Vec<String>,
    },

    /// Sent if a player has bluffed
    ExposePlayer {
        /// id of the player who is exposed
        player: PlayerId,
        /// the current wallet the player has
        wallet: Wallet,
    },

    /// Sent if an animal is drawn that results in inflation
    Inflation(Money),
}

/// After an auction has finished it is described by a kind
#[pyclass()]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuctionKind {
    /// No one has placed a bid. The host will receive the animal
    NoBiddings {
        /// the player who started the auction
        host_id: PlayerId,
        /// the animal the host receives
        animal: Animal,
    },

    /// Information about the bids that have been placed
    NormalAuction {
        /// contains a list of bids that have been made during the auction
        rounds: AuctionRound,
        /// the player who has to spent money and receives the animal
        from: PlayerId,
        /// the player who receives the money (the seller)
        to: PlayerId,
        /// contains the information about the cash flow after the current auction
        money_transfer: MoneyTransfer,
    },
}

/// Information about what money has been transferred after an auction
#[pyclass()]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MoneyTransfer {
    /// only the number of cards/ bills that have been exchanged and the minimum value to be payed are exposed to players not participating in the current money transfer
    Public {
        card_amount: usize,
        min_value: Value,
    },

    /// players participating in the money transfer will receive full information about what cards/ bills has been exchanged
    Private { amount: Vec<Money> },
}

/// Information about what money has been transferred after a trade
#[pyclass()]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MoneyTrade {
    /// only the number of cards/ bills that have been exchanged are exposed to players not participating in the current money trade
    Public {
        /// the number of money cards the challenger proposed
        challenger_card_offer: usize,
        /// the number of cards the opponent placed in the trade or [None] if he accepts the offer
        opponent_card_offer: Option<usize>,
    },

    /// players participating in the money trade will receive full information about what cards/ bills has been exchanged
    Private {
        /// the exact money that has been transfered from challenger to opponent
        challenger_card_offer: Vec<Money>,
        /// the exact cards that have been placed by the opponent or [None] if he accept the offer
        opponent_card_offer: Option<Vec<Money>>,
    },
}
