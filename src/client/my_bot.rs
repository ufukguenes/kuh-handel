use std::sync::Arc;

use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney, TradeOffer,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{AuctionRound, GameUpdate};
use crate::model::{
    money::{money::Money, value::Value},
    player::{base_player::PlayerId, player_actions::base_player_actions::PlayerActions},
};
pub struct MyBot {}

impl PlayerActions for MyBot {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        todo!()
    }

    fn _trade(&mut self) -> InitialTrade {
        todo!()
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        todo!()
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        todo!()
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        todo!()
    }

    fn _receive_from_player(&mut self, player: &PlayerId, money: Vec<Money>) -> NoAction {
        todo!()
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        todo!()
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        todo!()
    }
}
