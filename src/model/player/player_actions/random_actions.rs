use crate::model::money::money::Money;
use crate::model::money::value::Value;
use crate::model::player::base_player::PlayerId;
use crate::model::player::player_actions::actions::{
    InitialTrade, TradeOffer, TradeOpponentDecision,
};
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use crate::model::player::player_actions::game_updates::Bidding;
use crate::player_actions::actions::{AuctionDecision, PlayerTurnDecision};
use crate::player_actions::game_updates::{AuctionRound, GameUpdate};
pub struct RandomPlayerActions {}

impl PlayerActions for RandomPlayerActions {
    fn provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        Bidding::Pass
    }

    fn draw_or_trade(&mut self) -> PlayerTurnDecision {
        PlayerTurnDecision::Draw
    }

    fn buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        AuctionDecision::Buy
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

    // ToDo: add the other actions -> the actual trade needs to be implemented (doing the attack as well as the counter bid)
}
