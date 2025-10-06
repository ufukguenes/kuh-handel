use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney, TradeOffer,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{AuctionRound, GameUpdate};
use crate::model::money::money::Money;
use crate::model::money::value::Value;
use crate::model::player::base_player::PlayerId;
use crate::model::player::player_actions::base_player_actions::PlayerActions;
pub struct RandomPlayerActions {}

impl PlayerActions for RandomPlayerActions {
    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        Bidding::Pass
    }

    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        PlayerTurnDecision::Draw
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        AuctionDecision::Buy
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
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

    fn _trade(&mut self) -> InitialTrade {
        todo!()
    }

    // ToDo: add the other actions -> the actual trade needs to be implemented (doing the attack as well as the counter bid)
}
