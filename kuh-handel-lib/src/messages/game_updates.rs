use serde::{Deserialize, Serialize};
use std::rc::Rc;

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

pub type Points = usize;

/// Information about the currently running auction
///
/// # Arguments
///
/// `host` - the player that hosts the current auction
/// `animal` - the animal that is auctioned off by the host
/// `bids` - the current bids that have been placed until now by all other players
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuctionRound {
    pub host: PlayerId,
    pub animal: Rc<Animal>,
    pub bids: Vec<(PlayerId, Bidding)>,
}

/// Information about a trade offer from another player
///
/// # Arguments
///
/// `challenger` - id of the player that initiated the trade
/// `animal` - The animal that is offered for the trade
/// `animal_count` - the number of animals that are going to be traded
/// `challenger_card_offer` - the number of cards/ bills the challenger has offered, the actual card values are hidden
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradeOffer {
    pub challenger: PlayerId,
    pub animal: Animal,
    pub animal_count: usize,
    pub challenger_card_offer: usize,
}

/// After each game event, all players are informed about what happened.
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameUpdate {
    /// Sent after an auction has finished.
    Auction(AuctionKind),

    /// Sent after a trade is finished
    ///
    /// # Arguments
    ///
    /// `challenger` - the player id who initiated the trade
    /// `opponent` - the player id who was chosen for the trade by the challenger
    /// `animal` - the animal that was traded
    /// `animal_count` - the number of animals that have been traded
    /// `receiver` - the player id of the player who has placed the larger amount of money or if the same amount was placed, the challenger
    /// `money_trade` - if the player this message is sent to is either the challenger or opponent it receives a Private MoneyTrade with more information about the trade, else the player receives a Public MoneyTrade
    Trade {
        challenger: PlayerId,
        opponent: PlayerId,
        animal: Animal,
        animal_count: usize,
        receiver: PlayerId,
        money_trade: MoneyTrade,
    },

    /// Sent after a game started
    ///
    /// # Arguments
    ///
    /// `wallet` - the wallet each player is handed out at the beginning of the game, with the initial amount of money
    /// `players_in_turn_order` - the players sorted by their id in the order they are going to play
    /// `animals` - the animals that exist in this game
    Start {
        wallet: Wallet,
        players_in_turn_order: Vec<PlayerId>,
        animals: Vec<AnimalSet>,
    },

    /// Sent after the game is finished
    ///
    /// # Arguments
    ///
    /// `ranking` - the points each player achieved in this game
    End { ranking: Vec<(PlayerId, Points)> },

    /// Sent if a player has bluffed
    ///
    /// # Arguments
    ///
    /// `player` - id of the player who is exposed
    /// `wallet` - the current wallet the player has
    ExposePlayer { player: PlayerId, wallet: Wallet },

    /// Sent if an animal is drawn that results in inflation
    Inflation(Money),
}

/// After an auction has finished it is described by a kind
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuctionKind {
    /// No one has placed a bid. The host will receive the animal
    NoBiddings { host_id: PlayerId, animal: Animal },

    /// Information about the bids that have been placed
    NormalAuction {
        rounds: AuctionRound,
        from: PlayerId,
        to: PlayerId,
        money_transfer: MoneyTransfer,
    },
}

/// Information about what money has been transferred after an auction
#[pyclass(unsendable)]
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
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MoneyTrade {
    /// only the number of cards/ bills that have been exchanged are exposed to players not participating in the current money trade
    Public {
        challenger_card_offer: usize,
        opponent_card_offer: Option<usize>,
    },

    /// players participating in the money trade will receive full information about what cards/ bills has been exchanged
    Private {
        challenger_card_offer: Vec<Money>,
        opponent_card_offer: Option<Vec<Money>>,
    },
}
