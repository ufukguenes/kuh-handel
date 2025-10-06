use crate::messages::game_updates::AnimalTradeCount;
use crate::model::{animals::Animal, money::money::Money, player::base_player::PlayerId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerTurnDecision {
    Draw,
    Trade(InitialTrade),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitialTrade {
    pub opponent: PlayerId,
    pub animal: Animal,
    pub animal_count: AnimalTradeCount,
    pub amount: Vec<Money>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TradeOpponentDecision {
    Accept,
    CounterOffer { amount: Vec<Money> },
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
