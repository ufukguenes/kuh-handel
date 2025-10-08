use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney, TradeOffer,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{AuctionRound, GameUpdate};
use crate::model::animals::Animal;
use crate::model::money::money::Money;
use crate::model::money::value::Value;
use crate::model::player::base_player::PlayerId;
use crate::model::player::player_actions::base_player_actions::PlayerActions;
pub struct RandomPlayerActions {}

// todo make a more useful random player
impl PlayerActions for RandomPlayerActions {
    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        Bidding::Pass
    }

    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        PlayerTurnDecision::Draw
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        AuctionDecision::Sell
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        NoAction::Ok
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        SendMoney::Amount(vec![Money::new(amount)])
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        TradeOpponentDecision::Accept
    }

    fn _trade(&mut self) -> InitialTrade {
        InitialTrade {
            opponent: PlayerId {
                name: "".to_string(),
            },
            animal: Animal::new(Value::new(0)),
            animal_count: 1,
            amount: vec![Money::new_usize(0)],
        }
    }

    // ToDo: add the other actions -> the actual trade needs to be implemented (doing the attack as well as the counter bid)
}
