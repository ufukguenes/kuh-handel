use std::collections::HashMap;
use std::usize;

use rand::seq::{IndexedRandom, IteratorRandom};
use rand::{Rng, SeedableRng, rng};
use rand_chacha::ChaCha8Rng;
use serde::de::value;

use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney, TradeOffer,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{AuctionRound, GameUpdate};
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
    rng: ChaCha8Rng,
}

impl MyBot {
    pub fn new(my_id: String, seed: u64) -> MyBot {
        MyBot {
            opponents: Vec::new(),
            my_animals: HashMap::new(),
            my_money: Wallet::new(HashMap::new()),
            my_id: PlayerId { name: my_id },
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub fn get_random_trade(&mut self) -> Option<InitialTrade> {
        let random_opponent = self.opponents.choose(&mut self.rng).unwrap();

        if let Some((&random_animal, &animal_count)) = self.my_animals.iter().choose(&mut self.rng)
        {
            let random_animal_count = self.rng.random_range(1..=animal_count) as usize;
            if let Some(value) = self.my_money.total_money() {
                let random_amount = self
                    .my_money
                    .propose_bill_combinations(value, false)
                    .get(0)
                    .unwrap()
                    .1
                    .clone();

                let trade_choice = InitialTrade {
                    opponent: random_opponent.clone(),
                    animal: random_animal,
                    animal_count: random_animal_count,
                    amount: random_amount,
                };

                return Some(trade_choice);
            }
        }
        None
    }

    pub fn get_highest_bid(bids: &Vec<(PlayerId, Bidding)>) -> Option<(&PlayerId, &Value)> {
        bids.iter()
            .rev()
            .filter_map(|(id, bid)| {
                if let Bidding::Bid(val) = bid {
                    Some((id, val))
                } else {
                    None
                }
            })
            .next()
    }
}

impl PlayerActions for MyBot {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        if let Some(random_trade) = self.get_random_trade() {
            return vec![
                PlayerTurnDecision::Draw,
                PlayerTurnDecision::Trade(random_trade),
            ]
            .choose(&mut self.rng)
            .unwrap()
            .clone();
        }
        PlayerTurnDecision::Draw
    }

    fn _trade(&mut self) -> InitialTrade {
        self.get_random_trade().unwrap()
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        let mut new_val = 1;
        if let Some((_, value)) = Self::get_highest_bid(&state.bids) {
            let temp = value.value() as f32 * 1.1;
            new_val = new_val + temp as usize;
        }
        let random_bid = Bidding::Bid(Value::new(new_val));

        return vec![Bidding::Pass, random_bid]
            .choose(&mut self.rng)
            .unwrap()
            .clone();
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        if let Some((_, bid_value)) = Self::get_highest_bid(&state.bids) {
            if let Some(my_value) = self.my_money.total_money() {
                if bid_value > &my_value {
                    return vec![AuctionDecision::Sell, AuctionDecision::Buy]
                        .choose(&mut self.rng)
                        .unwrap()
                        .clone();
                }
            }
        }
        AuctionDecision::Sell
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        let combination = self.my_money.propose_bill_combinations(amount, false);
        SendMoney::Amount(combination.get(0).unwrap().1.clone())
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        if let Some(min_payment) = self.my_money.get_min_payment() {
            let random_value = self
                .rng
                .random_range(0..=self.my_money.total_money().unwrap().value());
            let combination = self
                .my_money
                .propose_bill_combinations(Value::new(random_value), false);
            let counter_offer =
                TradeOpponentDecision::CounterOffer(combination.get(0).unwrap().1.clone());
            return vec![TradeOpponentDecision::Accept]
                .choose(&mut self.rng)
                .unwrap()
                .clone();
        }
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
            GameUpdate::Auction(_) => {}
            GameUpdate::Trade {
                challenger,
                opponent,
                animal,
                animal_count,
                receiver,
                money_trade,
            } => {}
        }

        NoAction::Ok
    }
}
