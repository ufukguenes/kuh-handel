use crate::model::{
    animals::Animal,
    money::{money::Money, value::Value},
    player::base_player::{Player, PlayerId},
    player::player_actions::game_updates::AnimalTradeCount,
};

pub enum PlayerTurnDecision {
    Draw,
    Trade {
        opponent: PlayerId,
        animal: Animal,
        animal_count: AnimalTradeCount,
        amount: Vec<Money>,
    },
}

pub struct TradeOffer {
    challenger: PlayerId,
    animal: Animal,
    animal_count: AnimalTradeCount,
    challenger_card_offer: usize,
}

pub enum AuctionAction {
    Buy,
    Sell,
}

pub enum TradeOpponentDecision {
    Accept,
    CounterOffer { amount: Vec<Money> },
}
