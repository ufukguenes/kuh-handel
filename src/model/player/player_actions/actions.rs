use serde::{Deserialize, Serialize};

use crate::model::{
    animals::Animal, money::money::Money, player::base_player::PlayerId,
    player::player_actions::game_updates::AnimalTradeCount,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerTurnDecision {
    Draw,
    Trade {
        opponent: PlayerId,
        animal: Animal,
        animal_count: AnimalTradeCount,
        amount: Vec<Money>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradeOffer {
    challenger: PlayerId,
    animal: Animal,
    animal_count: AnimalTradeCount,
    challenger_card_offer: usize,
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
