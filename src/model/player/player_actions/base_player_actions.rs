use crate::model::player::base_player::{
    AuctionAction, AuctionState, AuctionValue, FirstPhaseAction,
};

pub trait PlayerActions: Send + Sync {
    fn provide_bidding(&mut self, state: AuctionState) -> AuctionValue;
    fn draw_or_trade(&mut self) -> FirstPhaseAction;
    fn buy_or_sell(&mut self, state: AuctionState) -> AuctionAction;
}
