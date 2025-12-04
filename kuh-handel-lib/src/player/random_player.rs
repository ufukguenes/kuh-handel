use std::collections::BTreeMap;
use std::usize;

use rand::seq::{IndexedRandom, IteratorRandom};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{
    AuctionKind, AuctionRound, GameUpdate, MoneyTrade, MoneyTransfer, Points, TradeOffer,
};
use crate::player::player_error::PlayerError;

use crate::{
    animals::Animal, player::base_player::PlayerId, player::player_actions::PlayerActions,
};

use crate::Value;
use crate::player::wallet::Wallet;

// todo: should this wrap the base player and use the functions of that, here is already duplicate code
// yes we should,this also duplicates code form supervised player

// todo check if this bot maybe changes its state somewhere where the supervisor does not, this will go away when i just remove the code duplication
#[derive(Debug, Clone)]
pub struct RandomPlayerActions {
    opponents: Vec<PlayerId>,
    owned_animals: BTreeMap<Animal, usize>,
    wallet: Wallet,
    id: PlayerId,
    rng: ChaCha8Rng,
    final_ranking: Vec<(PlayerId, Points)>,
    has_passed_this_auction_round: bool,
}

impl RandomPlayerActions {
    pub fn new(id: String, seed: u64) -> RandomPlayerActions {
        RandomPlayerActions {
            opponents: Vec::new(),
            owned_animals: BTreeMap::new(),
            wallet: Wallet::new(BTreeMap::new()),
            id: id,
            rng: ChaCha8Rng::seed_from_u64(seed),
            final_ranking: Vec::new(),
            has_passed_this_auction_round: false,
        }
    }

    pub fn get_random_trade(&mut self) -> Option<InitialTrade> {
        let random_opponent = self.opponents.choose(&mut self.rng)?;

        let (&random_animal, &animal_count) = self.owned_animals.iter().choose(&mut self.rng)?;

        let value = self.wallet.total_money();
        let random_amount = self
            .wallet
            .propose_bill_combinations(value, false)
            .get(0)?
            .1
            .clone();

        let trade_choice = InitialTrade {
            opponent: random_opponent.clone(),
            animal: random_animal,
            animal_count: animal_count,
            amount: random_amount,
        };

        return Some(trade_choice);
    }

    pub fn final_ranking(&self) -> &Vec<(PlayerId, Points)> {
        &self.final_ranking
    }

    pub fn get_highest_bid(bids: &Vec<(PlayerId, Bidding)>) -> Option<(&PlayerId, &Value)> {
        match bids.iter().max_by_key(|(_, bid)| bid) {
            Some((player_id, Bidding::Bid(value))) => Some((player_id, value)),
            Some(&(_, Bidding::Pass())) => None,
            None => None,
        }
    }

    pub fn add_animals(&mut self, animal: &Animal, count: usize) {
        self.owned_animals
            .entry(*animal)
            .and_modify(|current| *current += count)
            .or_insert(count);
    }

    pub fn remove_animals(&mut self, animal: &Animal, count: usize) -> Result<(), PlayerError> {
        let backup_animals = self.owned_animals.clone();
        let current_count = self.owned_animals.get_mut(animal);
        match current_count {
            Some(current_count) => {
                let res: isize = *current_count as isize - count as isize;
                if res > 0 {
                    *current_count -= count;
                } else if *current_count == 0 || res == 0 {
                    self.owned_animals.remove(animal);
                } else {
                    self.owned_animals = backup_animals;
                    return Result::Err(PlayerError::AnimalsNotAvailable);
                }
            }
            None => {
                self.owned_animals = backup_animals;
                return Result::Err(PlayerError::AnimalsNotAvailable);
            }
        }

        Ok(())
    }
}

impl PlayerActions for RandomPlayerActions {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        if let Some(random_trade) = self.get_random_trade() {
            return vec![
                PlayerTurnDecision::Draw(),
                PlayerTurnDecision::Trade(random_trade),
            ]
            .choose(&mut self.rng)
            .unwrap()
            .clone();
        }
        PlayerTurnDecision::Draw()
    }

    fn _trade(&mut self) -> InitialTrade {
        self.get_random_trade().unwrap() // todo remove this unwrap if we return option 
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        let mut new_val = 1;
        if let Some((_, value)) = Self::get_highest_bid(&state.bids) {
            let temp = *value as f32 * 1.1;
            new_val = new_val + temp as usize;
        }
        let random_bid = Bidding::Bid(new_val);

        let decision = vec![Bidding::Pass(), random_bid]
            .choose(&mut self.rng)
            .unwrap()
            .clone();

        match decision {
            Bidding::Pass() => {
                self.has_passed_this_auction_round = true;
                decision
            }
            Bidding::Bid(_) => {
                if self.has_passed_this_auction_round {
                    return Bidding::Pass();
                }
                decision
            }
        }
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        if let Some((_, bid_value)) = Self::get_highest_bid(&state.bids) {
            let my_value = self.wallet.total_money();
            if bid_value > &my_value {
                return vec![AuctionDecision::Sell, AuctionDecision::Buy]
                    .choose(&mut self.rng)
                    .unwrap()
                    .clone();
            }
        }
        AuctionDecision::Sell
    }

    fn _send_money_to_player(&mut self, _player: &PlayerId, amount: Value) -> SendMoney {
        if let Some(combination) = self.wallet.propose_bill_combinations(amount, false).get(0) {
            return SendMoney::Amount(combination.1.clone());
        }
        SendMoney::WasBluff()
    }

    fn _respond_to_trade(&mut self, _offer: TradeOffer) -> TradeOpponentDecision {
        let random_value = self.rng.random_range(0..=self.wallet.total_money());
        let combination = self.wallet.propose_bill_combinations(random_value, false);
        match combination.get(0) {
            Some((_, combination)) => {
                let counter_offer = TradeOpponentDecision::CounterOffer(combination.clone());
                return vec![TradeOpponentDecision::Accept(), counter_offer]
                    .choose(&mut self.rng)
                    .unwrap()
                    .clone();
            }
            None => TradeOpponentDecision::Accept(),
        }
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        match update.clone() {
            GameUpdate::Auction(auction_kind) => {
                self.has_passed_this_auction_round = false;

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
                    } => match money_transfer {
                        MoneyTransfer::Private { amount } => {
                            if self.id == from {
                                let _ = self.wallet.withdraw(&amount);
                                self.add_animals(&rounds.animal, 1);
                            } else if self.id == to {
                                self.wallet.deposit(&amount);
                            }
                        }
                        MoneyTransfer::Public {
                            card_amount: _,
                            min_value: _,
                        } => {}
                    },
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
                if player_id == challenger || player_id == opponent {
                    if player_id == receiver {
                        self.add_animals(&animal, animal_count);
                    } else {
                        let _ = self.remove_animals(&animal, animal_count);
                    }
                }
                match money_trade {
                    MoneyTrade::Private {
                        challenger_card_offer,
                        opponent_card_offer,
                    } => {
                        if player_id == challenger {
                            let _ = self.wallet.withdraw(&challenger_card_offer);
                            opponent_card_offer.map(|amount| self.wallet.deposit(&amount));
                        } else {
                            opponent_card_offer.map(|amount| self.wallet.withdraw(&amount));
                            self.wallet.deposit(&challenger_card_offer);
                        }
                    }
                    MoneyTrade::Public {
                        challenger_card_offer: _,
                        opponent_card_offer: _,
                    } => {}
                }
            }
            GameUpdate::Start {
                wallet,
                players_in_turn_order,
                animals: _,
            } => {
                self.opponents = players_in_turn_order
                    .iter()
                    .cloned()
                    .filter(|p| *p != self.id)
                    .collect();
                self.wallet = wallet;
                self.owned_animals = BTreeMap::new();
            }
            GameUpdate::End { ranking } => {
                self.final_ranking = ranking;
            }
            GameUpdate::Inflation(value) => {
                self.wallet.add_money(value);
            }
            GameUpdate::ExposePlayer {
                player: _,
                wallet: _,
            } => {}
        }

        NoAction::Ok
    }
}
