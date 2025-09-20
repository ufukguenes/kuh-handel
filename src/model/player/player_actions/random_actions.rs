use crate::model::player::base_player::{
    AuctionAction, AuctionState, AuctionValue, FirstPhaseAction,
};
use crate::model::player::player_actions::base_player_actions::PlayerActions;
pub struct RandomPlayerActions {}

impl PlayerActions for RandomPlayerActions {
    fn provide_bidding(&mut self, state: AuctionState) -> AuctionValue {
        AuctionValue::Pass
    }

    fn draw_or_trade(&mut self) -> FirstPhaseAction {
        FirstPhaseAction::Draw
    }

    fn buy_or_sell(&mut self, state: AuctionState) -> AuctionAction {
        AuctionAction::Buy
    }

    // ToDo: add the other actions -> the actual trade needs to be implemented (doing the attack as well as the counter bid)
}
