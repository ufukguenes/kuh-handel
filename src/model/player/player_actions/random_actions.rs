use std::collections::HashMap;
use std::usize;

use rand::seq::{IndexedRandom, IteratorRandom};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney, TradeOffer,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{
    AuctionKind, AuctionRound, GameUpdate, MoneyTrade, MoneyTransfer,
};
use crate::model::game_errors::GameError;
use crate::model::player::base_player::Player;
use crate::model::{
    animals::Animal,
    money::value::Value,
    money::wallet::Wallet,
    player::{base_player::PlayerId, player_actions::base_player_actions::PlayerActions},
};

pub struct RandomPlayerActions {
    opponents: Vec<PlayerId>,
    owned_animals: HashMap<Animal, usize>,
    wallet: Wallet,
    id: PlayerId,
    rng: ChaCha8Rng,
}

impl RandomPlayerActions {
    pub fn new(my_id: String, seed: u64) -> RandomPlayerActions {
        RandomPlayerActions {
            opponents: Vec::new(),
            owned_animals: HashMap::new(),
            wallet: Wallet::new(HashMap::new()),
            id: PlayerId { name: my_id },
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub fn get_random_trade(&mut self) -> Option<InitialTrade> {
        let random_opponent = self.opponents.choose(&mut self.rng).unwrap();

        if let Some((&random_animal, &animal_count)) =
            self.owned_animals.iter().choose(&mut self.rng)
        {
            let random_animal_count = self.rng.random_range(1..=animal_count) as usize;
            if let Some(value) = self.wallet.total_money() {
                let random_amount = self
                    .wallet
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

    pub fn add_animals(&mut self, animal: &Animal, count: usize) {
        self.owned_animals
            .entry(*animal)
            .and_modify(|current| *current += count)
            .or_insert(count);
    }

    pub fn remove_animals(&mut self, animal: &Animal, count: usize) -> Result<(), GameError> {
        let current_count = self.owned_animals.get_mut(animal);
        match current_count {
            Some(current_count) => {
                if *current_count - count > 0 {
                    *current_count -= count;
                } else if *current_count - count > 0 {
                    self.owned_animals.remove(animal);
                } else {
                    return Result::Err(GameError::AnimalsNotAvailable);
                }
            }
            None => return Result::Err(GameError::AnimalsNotAvailable),
        }

        Ok(())
    }
}

impl PlayerActions for RandomPlayerActions {
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
        println!("my animal: {:?}", self.owned_animals);
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
            if let Some(my_value) = self.wallet.total_money() {
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
        if let Some(combination) = self.wallet.propose_bill_combinations(amount, false).get(0) {
            return SendMoney::Amount(combination.1.clone());
        }
        SendMoney::WasBluff
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        if let Some(min_payment) = self.wallet.get_min_payment() {
            let random_value = self
                .rng
                .random_range(0..=self.wallet.total_money().unwrap().value());
            let combination = self
                .wallet
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
        match update.clone() {
            GameUpdate::Auction(auction_kind) => {
                match auction_kind {
                    AuctionKind::NoBiddings { host_id, animal } => {
                        if host_id == self.id {
                            self.add_animals(&animal, 1);
                        }
                    }
                    AuctionKind::NormalAuction {
                        rounds,
                        from,
                        to,
                        money_transfer,
                    } => {
                        match money_transfer {
                            // check if what animal, not necessary to check if host, because is checked with from to
                            MoneyTransfer::Private { amount } => {
                                if self.id == from {
                                    self.wallet.withdraw(&amount);
                                    self.add_animals(&rounds.animal, 1);
                                } else if self.id == to {
                                    self.wallet.deposit(&amount);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            GameUpdate::Trade {
                challenger,
                opponent,
                animal,
                animal_count,
                receiver,
                money_trade,
            } => {
                let animal_count: usize = animal_count.clone();
                let player_id = self.id.clone();
                if (player_id == challenger || player_id == opponent) && player_id == receiver {
                    self.add_animals(&animal, animal_count);
                } else if (player_id == challenger || player_id == opponent)
                    && player_id != receiver
                {
                    self.remove_animals(&animal, animal_count);
                }
                match money_trade {
                    MoneyTrade::Private {
                        challenger_card_offer,
                        opponent_card_offer,
                    } => {
                        if player_id == challenger {
                            self.wallet.withdraw(&challenger_card_offer);
                            opponent_card_offer.map(|amount| self.wallet.deposit(&amount));
                        } else {
                            opponent_card_offer.map(|amount| self.wallet.withdraw(&amount));
                            self.wallet.deposit(&challenger_card_offer);
                        }
                    }
                    _ => {}
                }
            }

            GameUpdate::Start {
                wallet,
                players_in_turn_order,
                animals,
            } => {
                players_in_turn_order
                    .clone()
                    .retain(|p| p.name != self.id.name);
                self.opponents = players_in_turn_order;
                self.wallet = wallet;
                self.owned_animals = HashMap::new();
            }

            _ => {}
        }

        NoAction::Ok
    }
}
