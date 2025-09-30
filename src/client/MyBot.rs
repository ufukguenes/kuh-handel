use crate::model::player::player_actions::base_player_actions::PlayerActions;

pub struct MyBot {}

impl PlayerActions for MyBot {
    fn draw_or_trade(
        &mut self,
    ) -> crate::model::player::player_actions::actions::PlayerTurnDecision {
        todo!()
    }

    fn provide_bidding(
        &mut self,
        state: crate::model::player::player_actions::game_updates::AuctionRound,
    ) -> crate::model::player::player_actions::actions::AuctionValue {
        todo!()
    }

    fn buy_or_sell(
        &mut self,
        state: crate::model::player::player_actions::game_updates::AuctionRound,
    ) -> crate::model::player::player_actions::actions::AuctionAction {
        todo!()
    }

    fn receive_game_update(
        &mut self,
        update: crate::model::player::player_actions::game_updates::GameUpdate,
    ) {
        todo!()
    }
}
