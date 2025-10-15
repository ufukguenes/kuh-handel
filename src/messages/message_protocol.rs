use std::fmt::Display;

use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney, TradeOffer,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{AuctionRound, GameUpdate};
use crate::model::{money::value::Value, player::base_player::PlayerId};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum StateMessage {
    DrawOrTrade,
    Trade,
    ProvideBidding { state: AuctionRound },
    BuyOrSell { state: AuctionRound },
    SendMoney { player_id: PlayerId, amount: Value },
    RespondToTrade { offer: TradeOffer },
    GameUpdate { update: GameUpdate },
}

impl Display for StateMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateMessage::DrawOrTrade => write!(f, "DrawOrTrade"),
            StateMessage::Trade => write!(f, "Trade"),
            StateMessage::ProvideBidding { .. } => write!(f, "ProvideBidding"),
            StateMessage::BuyOrSell { .. } => write!(f, "BuyOrSell"),
            StateMessage::SendMoney { .. } => write!(f, "SendMoney"),
            StateMessage::RespondToTrade { .. } => write!(f, "RespondToTrade"),
            StateMessage::GameUpdate { .. } => write!(f, "GameUpdate"),
        }
    }
}
