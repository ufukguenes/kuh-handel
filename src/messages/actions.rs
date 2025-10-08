use crate::messages::game_updates::AnimalTradeCount;
use crate::messages::message_protocol::ActionMessage;
use crate::model::{animals::Animal, money::money::Money, player::base_player::PlayerId};
use serde::{Deserialize, Serialize};

pub trait FromActionMessage: Sized {
    fn extract(action: ActionMessage) -> Self;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NoAction {
    Ok,
}
impl FromActionMessage for NoAction {
    fn extract(action: ActionMessage) -> Self {
        match action {
            ActionMessage::NoAction { decision } => decision,
            _ => panic!("Expected ActionMessage::NoAction"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerTurnDecision {
    Draw,
    Trade(InitialTrade),
}

impl FromActionMessage for PlayerTurnDecision {
    fn extract(action: ActionMessage) -> Self {
        match action {
            ActionMessage::PlayerTurnDecision { decision } => decision,
            _ => panic!("Expected ActionMessage::PlayerTurnDecision"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitialTrade {
    pub opponent: PlayerId,
    pub animal: Animal,
    pub animal_count: AnimalTradeCount,
    pub amount: Vec<Money>,
}

impl FromActionMessage for InitialTrade {
    fn extract(action: ActionMessage) -> Self {
        match action {
            ActionMessage::InitialTrade { decision } => decision,
            _ => panic!("Expected ActionMessage::InitialTrade"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradeOffer {
    pub challenger: PlayerId,
    pub animal: Animal,
    pub animal_count: AnimalTradeCount,
    pub challenger_card_offer: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuctionDecision {
    Buy,
    Sell,
}
impl FromActionMessage for AuctionDecision {
    fn extract(action: ActionMessage) -> Self {
        match action {
            ActionMessage::AuctionDecision { decision } => decision,
            _ => panic!("Expected ActionMessage::AuctionDecision"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TradeOpponentDecision {
    Accept,
    CounterOffer { amount: Vec<Money> },
}

impl FromActionMessage for TradeOpponentDecision {
    fn extract(action: ActionMessage) -> Self {
        match action {
            ActionMessage::TradeOpponentDecision { decision } => decision,
            _ => panic!("Expected ActionMessage::TradeOpponentDecision"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SendMoney {
    WasBluff,
    Amount(Vec<Money>),
}

impl FromActionMessage for SendMoney {
    fn extract(action: ActionMessage) -> Self {
        match action {
            ActionMessage::SendMoney { decision } => decision,
            _ => panic!("Expected ActionMessage::SendMoney"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub enum Bidding {
    Pass,
    Bid(Money),
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
    fn extract(action: ActionMessage) -> Self {
        match action {
            ActionMessage::Bidding { decision } => decision,
            _ => panic!("Expected ActionMessage::Bidding"),
        }
    }
}
