use std::sync::Arc;

use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, PlayerTurnDecision, TradeOffer, TradeOpponentDecision,
};
use crate::messages::game_updates::{AuctionRound, GameUpdate};
use crate::model::{
    money::{money::Money, value::Value},
    player::{base_player::PlayerId, player_actions::base_player_actions::PlayerActions},
};
pub struct MyBot {}

impl PlayerActions for MyBot {
    fn draw_or_trade(&mut self) -> PlayerTurnDecision {
        PlayerTurnDecision::Draw
    }

    fn provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        Bidding::Pass
    }

    fn buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        AuctionDecision::Buy
    }

    fn receive_game_update(&mut self, update: GameUpdate) {
        println!("received game update");
    }

    fn send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> Vec<Money> {
        vec![Money::new_u32(0)]
    }

    fn receive_from_player(&mut self, player: &PlayerId, money: Vec<Money>) {}

    fn respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        TradeOpponentDecision::Accept
    }

    fn trade(&mut self) -> InitialTrade {
        todo!()
    }
}
