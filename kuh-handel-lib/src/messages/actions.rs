use crate::messages::message_protocol::ActionMessage;
use crate::{Money, Value};
use crate::{animals::Animal, player::base_player::PlayerId};
use serde::{Deserialize, Serialize};
use tracing::error;

pub trait FromActionMessage: Sized {
    fn extract(action: ActionMessage) -> Option<Self>;
}
// todo switch these panics to results, so that when client sends wrong response game doesnt crash
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NoAction {
    Ok,
}
impl FromActionMessage for NoAction {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::NoAction { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::NoAction");
                None
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerTurnDecision {
    Draw,
    Trade(InitialTrade),
}

impl FromActionMessage for PlayerTurnDecision {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::PlayerTurnDecision { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::PlayerTurnDecision");
                None
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitialTrade {
    pub opponent: PlayerId,
    pub animal: Animal,
    pub animal_count: usize,
    pub amount: Vec<Money>,
}

impl FromActionMessage for InitialTrade {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::InitialTrade { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::InitialTrade");
                None
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradeOffer {
    pub challenger: PlayerId,
    pub animal: Animal,
    pub animal_count: usize,
    pub challenger_card_offer: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuctionDecision {
    Buy,
    Sell,
}
impl FromActionMessage for AuctionDecision {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::AuctionDecision { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::AuctionDecision");
                None
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TradeOpponentDecision {
    Accept,
    CounterOffer(Vec<Money>),
}

impl FromActionMessage for TradeOpponentDecision {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::TradeOpponentDecision { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::TradeOpponentDecision");
                None
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SendMoney {
    WasBluff,
    Amount(Vec<Money>),
}

impl FromActionMessage for SendMoney {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::SendMoney { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::SendMoney");
                None
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub enum Bidding {
    Pass,
    Bid(Value),
}

impl PartialEq for Bidding {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Bidding::Pass, Bidding::Pass) => true,
            (Bidding::Pass, _) => false,
            (_, Bidding::Pass) => false,
            (Bidding::Bid(a), Bidding::Bid(b)) => a == b,
        }
    }
}

impl Ord for Bidding {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Bidding::Pass, Bidding::Pass) => std::cmp::Ordering::Equal,
            (Bidding::Pass, _) => std::cmp::Ordering::Less,
            (_, Bidding::Pass) => std::cmp::Ordering::Greater,
            (Bidding::Bid(a), Bidding::Bid(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for Bidding {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl FromActionMessage for Bidding {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::Bidding { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::Bidding");
                None
            }
        }
    }
}
