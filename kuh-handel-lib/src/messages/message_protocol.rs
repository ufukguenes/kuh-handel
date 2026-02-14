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
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ActionMessage {
    PlayerTurnDecision { decision: PlayerTurnDecision },
    InitialTrade { decision: InitialTrade },
    Bidding { decision: Bidding },
    AuctionDecision { decision: AuctionDecision },
    SendMoney { decision: SendMoney },
    TradeOpponentDecision { decision: TradeOpponentDecision },
    NoAction { decision: NoAction },
}

/// State Messages are sent to a player describing a state of the current game which leads to an action
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum StateMessage {
    DrawOrTrade(),
    Trade(),
    ProvideBidding { state: AuctionRound },
    BuyOrSell { state: AuctionRound },
    SendMoney { player_id: PlayerId, amount: Value },
    RespondToTrade { offer: TradeOffer },
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
