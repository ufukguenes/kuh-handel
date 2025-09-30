use serde::{Deserialize, Serialize};

use crate::model::{
    animals::Animal, money::money::Money, player::base_player::PlayerId,
    player::player_actions::game_updates::AnimalTradeCount,
};

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
