use crate::model::{
    animals::Animal,
    money::{money::Money, value::Value},
    player::base_player::{Player, PlayerId},
};

pub enum FirstPhaseAction {
    Draw,
    Trade {
        opponent: PlayerId,
        animal: Animal,
        amount: Vec<Money>,
    },
}

pub enum AuctionAction {
    Buy,
    Sell,
}

pub enum AuctionValue {
    Bidding(Money),
    Pass,
}
