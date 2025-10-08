use std::collections::HashMap;

use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney, TradeOffer,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{AnimalTradeCount, AuctionRound, GameUpdate};
use crate::model::{
    animals::Animal,
    money::wallet::Wallet,
    money::{money::Money, value::Value},
    player::{base_player::PlayerId, player_actions::base_player_actions::PlayerActions},
};
pub struct MyBot {
    opponents: Vec<PlayerId>,
    my_animals: HashMap<Animal, usize>,
    my_money: Wallet,
    my_id: PlayerId,
}

impl MyBot {
    pub fn new(my_id: String) -> MyBot {
        MyBot {
            opponents: Vec::new(),
            my_animals: HashMap::new(),
            my_money: Wallet::new(HashMap::new()),
            my_id: PlayerId { name: my_id },
        }
    }
}

impl PlayerActions for MyBot {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        PlayerTurnDecision::Draw
    }

    fn _trade(&mut self) -> InitialTrade {
        InitialTrade {
            opponent: self.opponents.get(0).unwrap().clone(),
            animal: self.my_animals.keys().next().unwrap().clone(),
            animal_count: AnimalTradeCount::One,
            amount: vec![Money::new_usize(100), Money::new_usize(100)],
        }
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        Bidding::Pass
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        AuctionDecision::Sell
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        SendMoney::Amount(vec![Money::new(amount)])
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        TradeOpponentDecision::Accept
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        match update {
            GameUpdate::Start {
                wallet,
                players_in_turn_order,
                animals,
            } => {
                players_in_turn_order
                    .clone()
                    .retain(|p| p.name != self.my_id.name);
                self.opponents = players_in_turn_order;
                self.my_money = wallet;
                self.my_animals = HashMap::new();
            }
            GameUpdate::End { ranking } => {}
            GameUpdate::ExposePlayer { player, wallet } => {}
            GameUpdate::Auction {
                rounds,
                from,
                to,
                money_transfer,
            } => todo!(),
            GameUpdate::Trade {
                challenger,
                opponent,
                animal,
                animal_count,
                receiver,
                money_trade,
            } => todo!(),
        }

        NoAction::Ok
    }
}
