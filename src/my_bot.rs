use std::collections::HashMap;
use std::sync::Arc;

use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney, TradeOffer,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{AnimalTradeCount, AuctionRound, GameUpdate};
use crate::model::animals::Animal;
use crate::model::money::wallet::Wallet;
use crate::model::{
    money::{money::Money, value::Value},
    player::{base_player::PlayerId, player_actions::base_player_actions::PlayerActions},
};
pub struct MyBot {
    opponents: Vec<PlayerId>,
    my_animals: Vec<Animal>,
    my_money: Wallet,
    my_id: PlayerId,
}

impl MyBot {
    pub fn new(my_id: String) -> MyBot {
        MyBot {
            opponents: Vec::new(),
            my_animals: Vec::new(),
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
            animal: self.my_animals.get(0).unwrap().clone(),
            animal_count: AnimalTradeCount::One,
            amount: vec![Money::new_u32(100), Money::new_u32(100)],
        }
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        Bidding::Pass
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        AuctionDecision::Sell
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        SendMoney {
            amount: vec![Money::new(amount)],
        }
    }

    fn _receive_from_player(&mut self, player: &PlayerId, money: Vec<Money>) -> NoAction {
        NoAction::Ok
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        TradeOpponentDecision::Accept
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        match update {
            GameUpdate::Auction { rounds, transfer } => {}
            GameUpdate::Trade {
                challenger,
                opponent,
                animal,
                animal_count,
                challenger_card_offer,
                opponent_card_offer,
                receiver,
            } => {}
            GameUpdate::Start {
                wallet,
                players_in_turn_order,
                animals,
            } => {
                players_in_turn_order
                    .clone()
                    .retain(|p| p.name != self.my_id.name);
                self.opponents = players_in_turn_order;
                println!("opponents{:?}", self.opponents);
                self.my_money = wallet;
                self.my_animals = Vec::new();
            }
            GameUpdate::End { ranking } => {}
            GameUpdate::ExposePlayer { player, wallet } => {}
        }

        NoAction::Ok
    }
}
