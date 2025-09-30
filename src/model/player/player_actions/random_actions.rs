use crate::model::player::player_actions::base_player_actions::PlayerActions;
use crate::player_actions::actions::{AuctionAction, AuctionValue, FirstPhaseAction};
use crate::player_actions::game_updates::{AuctionRound, GameUpdate};
pub struct RandomPlayerActions {}

impl PlayerActions for RandomPlayerActions {
    fn provide_bidding(&mut self, state: AuctionRound) -> AuctionValue {
        AuctionValue::Pass
    }

    fn draw_or_trade(&mut self) -> FirstPhaseAction {
        FirstPhaseAction::Draw
    }

    fn buy_or_sell(&mut self, state: AuctionRound) -> AuctionAction {
        AuctionAction::Buy
    }

    fn receive_game_update(&mut self, update: super::game_updates::GameUpdate) {
        todo!()
    }

    // ToDo: add the other actions -> the actual trade needs to be implemented (doing the attack as well as the counter bid)
}
