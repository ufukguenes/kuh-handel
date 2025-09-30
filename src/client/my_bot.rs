use crate::model::{
    money::{money::Money, value::Value},
    player::{
        base_player::PlayerId,
        player_actions::{
            actions::{
                AuctionDecision, InitialTrade, PlayerTurnDecision, TradeOffer,
                TradeOpponentDecision,
            },
            base_player_actions::PlayerActions,
            game_updates::{AuctionRound, Bidding, GameUpdate},
        },
    },
};

pub struct MyBot {}

impl PlayerActions for MyBot {
    fn draw_or_trade(&mut self) -> PlayerTurnDecision {
        todo!()
    }

    fn provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        todo!()
    }

    fn buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        todo!()
    }

    fn receive_game_update(&mut self, update: GameUpdate) {
        todo!()
    }

    fn send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> Vec<Money> {
        todo!()
    }

    fn receive_from_player(&mut self, player: &PlayerId, money: Vec<Money>) {
        todo!()
    }

    fn respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        todo!()
    }

    fn trade(&mut self) -> InitialTrade {
        todo!()
    }
}
