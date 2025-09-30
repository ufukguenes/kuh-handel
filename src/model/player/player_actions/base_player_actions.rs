use crate::player_actions::actions::{AuctionAction, AuctionValue, FirstPhaseAction};
use crate::player_actions::game_updates::{AuctionRound, GameUpdate};
pub trait PlayerActions: Send + Sync {
    /// If it is a players turn, it must decide whether to draw a card or to trade with an other player.
    /// 1. draw
    ///     open card
    ///     start bidding
    ///     decided if to buy yourself
    fn draw_or_trade(&mut self) -> FirstPhaseAction;

    fn provide_bidding(&mut self, state: AuctionRound) -> AuctionValue;
    fn buy_or_sell(&mut self, state: AuctionRound) -> AuctionAction;

    fn receive_game_update(&mut self, update: GameUpdate);
}
