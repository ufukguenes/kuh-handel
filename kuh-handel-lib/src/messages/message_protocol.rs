use std::fmt::Display;

use crate::Value;
use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{AuctionRound, GameUpdate, TradeOffer};
use crate::player::base_player::PlayerId;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// Actions a player can make after receiving a state message
#[pyclass()]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ActionMessage {
    /// At the beginning of the turn the player can draw a card or trade with another player.
    PlayerTurnDecision { decision: PlayerTurnDecision },
    /// In the trading phase, the initial trade has to be send to setup the trade.
    InitialTrade { decision: InitialTrade },
    /// The player answers with the bidding in an auction round.
    Bidding { decision: Bidding },
    /// After the auction the host can decide to buy or to sell the animal.
    AuctionDecision { decision: AuctionDecision },
    /// Used to send money from one player to an other player (happens after the auction)
    SendMoney { decision: SendMoney },
    /// When the trade opponent receives a trade offer,
    /// the player must decide whether to accept or to counter it.
    TradeOpponentDecision { decision: TradeOpponentDecision },
    /// On each received game update, the players acknowledge it.
    NoAction { decision: NoAction },
}

/// State Messages are sent to a player describing a state of the current game which leads to an action
#[pyclass()]
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum StateMessage {
    /// At each turn the player is asked to draw or trade.
    /// Answer with [PlayerTurnDecision]
    DrawOrTrade(),
    /// In the trading a phase, the player is asked to trade.
    /// Answer with [InitialTrade]
    Trade(),
    /// During an auction the player has to bid.
    /// Answer with [Bidding]
    ProvideBidding { state: AuctionRound },
    /// The host of an auction is asked to buy or sell the current animal.
    /// Answer with [AuctionDecision]
    BuyOrSell { state: AuctionRound },
    /// After the auction one player is asked to pay the animal.
    /// Answer with [ActionMessage::SendMoney]
    SendMoney { player_id: PlayerId, amount: Value },
    /// The player is provided with a trade offer, and now has to answer the trade.
    /// Answer with [TradeOpponentDecision]
    RespondToTrade { offer: TradeOffer },
    /// After important game actions, the players are updated.
    /// Answer with [NoAction]
    GameUpdate { update: GameUpdate },
}

impl Display for StateMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateMessage::DrawOrTrade() => write!(f, "DrawOrTrade"),
            StateMessage::Trade() => write!(f, "Trade"),
            StateMessage::ProvideBidding { .. } => write!(f, "ProvideBidding"),
            StateMessage::BuyOrSell { .. } => write!(f, "BuyOrSell"),
            StateMessage::SendMoney { .. } => write!(f, "SendMoney"),
            StateMessage::RespondToTrade { .. } => write!(f, "RespondToTrade"),
            StateMessage::GameUpdate { .. } => write!(f, "GameUpdate"),
        }
    }
}
