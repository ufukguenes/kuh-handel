use core::num;
use std::{
    cmp,
    collections::{BTreeMap, BTreeSet},
    usize,
};

use crate::{
    Money, Value,
    animals::{Animal, AnimalSet},
    messages::{
        actions::{
            AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney,
            TradeOpponentDecision,
        },
        game_updates::{
            AuctionKind, AuctionRound, GameUpdate, MoneyTrade, MoneyTransfer, TradeOffer,
        },
    },
    player::{base_player::PlayerId, player_actions::PlayerActions, wallet::Wallet},
};
use float_ord::FloatOrd;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Default, Debug)]
pub struct ValueOwned {
    pub max: Value,
    pub min: Value,
}

impl ValueOwned {
    pub fn new(wallet: &Wallet) -> Self {
        let exact = wallet.total_money();
        ValueOwned {
            max: exact,
            min: exact,
        }
    }

    pub fn add(&mut self, value: Value) {
        self.max += value;
        self.min += value;
    }

    pub fn sub(&mut self, value: Value) {
        self.max = self.max.checked_sub(value).get_or_insert(0).clone();
        self.min = self.min.checked_sub(value).get_or_insert(0).clone();
    }

    pub fn value_at_step(&self, percentage: f32) -> Value {
        let value = (self.min as f32 + self.max as f32 * percentage) as usize;
        value
    }
}

pub struct SimplePlayer {
    id: PlayerId,
    opponents: BTreeMap<PlayerId, (BTreeMap<Animal, usize>, ValueOwned)>,
    wallet: Wallet,
    owned_animals: BTreeMap<Animal, usize>,
    all_animals: Vec<(Animal, usize)>,
    mean_points: usize,
    aggressiveness: f32,
    previous_subjective_values: Vec<f32>,
}

impl PlayerActions for SimplePlayer {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        if let Some((is_good, possible_trade)) = self.trade_helper() {
            if is_good {
                return PlayerTurnDecision::Trade(possible_trade);
            }
        }

        PlayerTurnDecision::Draw()
    }

    fn _trade(&mut self) -> InitialTrade {
        self.trade_helper().unwrap().1
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        let value_allowed_to_spend =
            (self.wallet.total_money() as f32 * self.aggressiveness) as usize;

        let empty_id = "".to_string();
        let (highest_bidder_id, &highest_bid) =
            Self::get_highest_bid(&state.bids).unwrap_or((&empty_id, &0));

        if highest_bidder_id == &self.id || highest_bid > value_allowed_to_spend {
            return Bidding::Pass();
        }

        let current_subj_value = Self::subjective_animal_value_for_player(
            &self.all_animals,
            &self.owned_animals,
            &state.animal,
            1,
        );

        let averaged_subj_values = self.average_subj_value_over_last(5);

        if Self::sqrt_points_for_player(&self.all_animals, &self.owned_animals)
            <= self.mean_points as f32
            || (highest_bid <= value_allowed_to_spend && averaged_subj_values < current_subj_value)
        {
            let bidding_rounded_to_next_ten = highest_bid / 10 * 10; // rounds down to next ten because integer division
            let my_bid = bidding_rounded_to_next_ten + 10;
            return Bidding::Bid(my_bid);
        }

        Bidding::Pass()
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        let value_allowed_to_spend =
            (self.wallet.total_money() as f32 * self.aggressiveness) as usize;
        let averaged_subj_values = self.average_subj_value_over_last(5);

        let (_, &highest_bid) = Self::get_highest_bid(&state.bids).unwrap_or((&"".to_string(), &0));

        let current_subj_value = Self::subjective_animal_value_for_player(
            &self.all_animals,
            &self.owned_animals,
            &state.animal,
            1,
        );

        if Self::sqrt_points_for_player(&self.all_animals, &self.owned_animals)
            <= self.mean_points as f32
            || (highest_bid <= value_allowed_to_spend && averaged_subj_values < current_subj_value)
        {
            return AuctionDecision::Buy;
        }

        AuctionDecision::Sell
    }

    fn _send_money_to_player(
        &mut self,
        player: &crate::player::base_player::PlayerId,
        amount: crate::Value,
    ) -> SendMoney {
        let bill_combination = self.get_bill_combination(amount);
        SendMoney::Amount(bill_combination)
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        let my_subj_value = Self::subjective_animal_value_for_player(
            &self.all_animals,
            &self.owned_animals,
            &offer.animal,
            offer.animal_count,
        );

        let opponent_subj_value = Self::subjective_animal_value_for_player(
            &self.all_animals,
            &self.opponents.get(&offer.challenger).unwrap().0,
            &offer.animal,
            offer.animal_count,
        );

        if my_subj_value >= opponent_subj_value {
            let value_allowed_to_spend =
                (self.wallet.total_money() as f32 * self.aggressiveness) as usize;

            let bill_combination = self.get_bill_combination(value_allowed_to_spend);
            TradeOpponentDecision::CounterOffer(bill_combination);
        }

        TradeOpponentDecision::Accept()
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        match update {
            GameUpdate::Auction(auction_kind) => self.handle_update_auction(auction_kind),
            GameUpdate::Trade {
                challenger,
                opponent,
                animal,
                animal_count,
                receiver,
                money_trade,
            } => self.handle_update_trade(
                challenger,
                opponent,
                animal,
                animal_count,
                receiver,
                money_trade,
            ),
            GameUpdate::Start {
                wallet,
                players_in_turn_order,
                animals,
            } => self.handle_update_start(wallet, players_in_turn_order, animals),
            GameUpdate::End { ranking } => println!("ranking: {:?}", ranking),
            GameUpdate::ExposePlayer { player, wallet } => {
                self.handle_update_expose(player, wallet)
            }
            GameUpdate::Inflation(money) => self.handle_update_inflation(money),
        }

        NoAction::Ok
    }
}

// handle updates
impl SimplePlayer {
    pub fn handle_update_auction(&mut self, auction_kind: AuctionKind) {
        let traded_animal;

        match auction_kind {
            AuctionKind::NoBiddings { host_id, animal } => {
                traded_animal = animal.clone();
                if self.id == host_id {
                    self.owned_animals
                        .entry(animal)
                        .and_modify(|count: &mut usize| *count += 1)
                        .or_insert(1);
                }
            }

            AuctionKind::NormalAuction {
                rounds,
                from,
                to,
                money_transfer,
            } => {
                traded_animal = *rounds.animal.clone();
                match money_transfer {
                    MoneyTransfer::Public {
                        card_amount: _,
                        min_value,
                    } => {
                        self.handle_exchange_payer(from, &rounds.animal, 1, min_value);
                        self.handle_exchange_seller(to, &rounds.animal, 0, min_value);
                    }
                    MoneyTransfer::Private { amount } => {
                        let total_value: usize = amount.iter().sum();
                        if self.id == from {
                            let could_pay = self.wallet.withdraw(&amount);
                            if could_pay.is_err() {
                                self.wallet = Wallet::default();
                                println!("miscalculated my own wallet")
                            }
                            self.owned_animals
                                .entry(*rounds.animal)
                                .and_modify(|count: &mut usize| *count += 1)
                                .or_insert(1);

                            self.handle_exchange_seller(to, &rounds.animal, 0, total_value);
                        } else {
                            self.wallet.deposit(&amount);
                            self.handle_exchange_payer(from, &rounds.animal, 1, total_value);
                        }
                    }
                }
            }
        }

        self.previous_subjective_values
            .push(Self::subjective_animal_value_for_player(
                &self.all_animals,
                &self.owned_animals,
                &traded_animal,
                1,
            ));
    }
    pub fn handle_update_trade(
        &mut self,
        challenger: PlayerId,
        opponent: PlayerId,
        animal: Animal,
        animal_count: usize,
        receiver: PlayerId,
        money_trade: MoneyTrade,
    ) {
        let looser = if receiver == challenger {
            opponent.clone()
        } else {
            challenger.clone()
        };

        match money_trade {
            MoneyTrade::Public {
                challenger_card_offer,
                opponent_card_offer,
            } => {
                self.handle_exchange_payer(receiver.clone(), &animal, animal_count, 0);
                self.handle_exchange_seller(looser, &animal, animal_count, 0);
            }
            MoneyTrade::Private {
                challenger_card_offer,
                opponent_card_offer,
            } => {
                let opponent_payed_value = match &opponent_card_offer {
                    Some(amount) => amount.iter().sum::<usize>(),
                    None => 0,
                };

                let total_difference: usize = (challenger_card_offer.iter().sum::<usize>() as isize
                    - opponent_payed_value as isize)
                    .abs() as usize;

                if self.id == receiver {
                    self.owned_animals
                        .entry(animal)
                        .and_modify(|count| *count += animal_count)
                        .or_insert(1);
                    self.handle_exchange_seller(looser, &animal, animal_count, total_difference);
                } else {
                    if animal_count > 0 {
                        let current_count = self.owned_animals.get(&animal).unwrap();
                        let new_count = cmp::min(
                            current_count
                                .checked_sub(animal_count)
                                .get_or_insert(0)
                                .clone(),
                            0,
                        );
                        self.owned_animals.insert(animal, new_count);
                    }
                    self.handle_exchange_payer(receiver, &animal, animal_count, total_difference);
                }

                if self.id == challenger {
                    let _ = self.wallet.withdraw(&challenger_card_offer);
                    opponent_card_offer.map(|amount| self.wallet.deposit(&amount));
                } else {
                    opponent_card_offer.map(|amount| self.wallet.withdraw(&amount));
                    self.wallet.deposit(&challenger_card_offer);
                }
            }
        }
    }

    pub fn handle_update_start(
        &mut self,
        wallet: Wallet,
        players_in_turn_order: Vec<PlayerId>,
        animals: Vec<AnimalSet>,
    ) {
        self.opponents = players_in_turn_order
            .iter()
            .cloned()
            .filter(|p| *p != self.id)
            .map(|p| (p, (BTreeMap::default(), ValueOwned::new(&wallet))))
            .collect();
        self.wallet = wallet;
        self.owned_animals = BTreeMap::default();

        let mut all_animals = Vec::default();
        for set in animals {
            all_animals.push((set.animal().clone(), set.occurrences()));
        }
        self.all_animals = all_animals;
        self.mean_points = self.estimate_average_points_per_player();
    }

    pub fn handle_update_expose(&mut self, player: PlayerId, wallet: Wallet) {
        self.opponents
            .entry(player)
            .and_modify(|(_, old_wallet)| *old_wallet = ValueOwned::new(&wallet));
    }

    pub fn handle_update_inflation(&mut self, money: Money) {
        self.wallet.add_money(money);
    }
}

// helper
impl SimplePlayer {
    pub fn new(id: String, aggressiveness: f32) -> Self {
        if 0.0 > aggressiveness || 1.0 < aggressiveness {
            panic!(
                "pick aggressiveness between 0 and 1, currently {}",
                aggressiveness
            )
        }
        SimplePlayer {
            id: id,
            opponents: BTreeMap::default(),
            wallet: Wallet::default(),
            owned_animals: BTreeMap::default(),
            all_animals: Vec::default(),
            mean_points: 0,
            aggressiveness: aggressiveness,
            previous_subjective_values: Vec::default(),
        }
    }

    pub fn new_from_seed(id: String, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let aggressiveness: f32 = rng.random_range(0.0..=1.0);
        SimplePlayer::new(id, aggressiveness)
    }

    pub fn handle_exchange_payer(
        &mut self,
        paying_player: PlayerId,
        animal: &Animal,
        animal_count: usize,
        min_value_payed: Value,
    ) {
        self.opponents
            .entry(paying_player)
            .and_modify(|(animals, value_owned)| {
                animals
                    .entry(*animal)
                    .and_modify(|count| *count += animal_count)
                    .or_insert(1);

                value_owned.sub(min_value_payed)
            });
    }

    pub fn handle_exchange_seller(
        &mut self,
        selling_player: PlayerId,
        animal: &Animal,
        animal_count: usize,
        min_value_payed: Value,
    ) {
        self.opponents
            .entry(selling_player)
            .and_modify(|(animals, value_owned)| {
                if animal_count > 0 {
                    let current_count = animals.get(animal).unwrap();
                    let new_count = cmp::min(
                        current_count
                            .checked_sub(animal_count)
                            .get_or_insert(0)
                            .clone(),
                        0,
                    );
                    animals.insert(*animal, new_count);
                }

                value_owned.add(min_value_payed)
            });
    }

    pub fn estimate_average_points_per_player(&mut self) -> usize {
        let mut total_points: usize = 0;
        let num_players = self.opponents.len() + 1;
        for (animal, _) in self.all_animals.iter() {
            total_points += animal.value()
        }
        let summed_card_points_per_player = total_points.div_ceil(num_players);
        let final_points_per_player = summed_card_points_per_player as f32
            * self.all_animals.len() as f32
            / num_players as f32;

        final_points_per_player as usize
    }

    pub fn sqrt_points_for_player(
        all_animals: &Vec<(Animal, usize)>,
        animals: &BTreeMap<Animal, usize>,
    ) -> f32 {
        let mut sqrt_points: f32 = 0.0;
        for (animal, occurrences) in all_animals.iter() {
            let my_animal_count = animals.get(animal);
            if my_animal_count.is_some() {
                let percentage_of_deck_owned =
                    *my_animal_count.unwrap() as f32 / *occurrences as f32;
                sqrt_points += percentage_of_deck_owned.sqrt() * animal.value() as f32;
            }
        }

        sqrt_points * animals.len() as f32
    }

    pub fn subjective_animal_value_for_player(
        all_animals: &Vec<(Animal, usize)>,
        animals: &BTreeMap<Animal, usize>,
        animal: &Animal,
        animal_count: usize,
    ) -> f32 {
        let points_without = Self::sqrt_points_for_player(all_animals, animals);
        let mut temp_animals = animals.clone();
        temp_animals
            .entry(*animal)
            .and_modify(|current_count| *current_count += animal_count)
            .or_insert(animal_count);

        let points_with = Self::sqrt_points_for_player(all_animals, &temp_animals);

        points_with - points_without
    }

    pub fn sorted_opponent_trades(&self) -> BTreeSet<(FloatOrd<f32>, PlayerId, Animal, usize)> {
        let mut sorted_trades: BTreeSet<(FloatOrd<f32>, PlayerId, Animal, usize)> =
            BTreeSet::default();

        let all_animals = &self.all_animals;
        let owned_animals = &self.owned_animals;

        for (id, (animals, _)) in self.opponents.iter() {
            for (animal, opponent_animal_count) in animals.iter() {
                if let Some(my_animal_count) = self.owned_animals.get(animal) {
                    let trade_count = cmp::min(my_animal_count, opponent_animal_count);

                    let opponent_value = Self::subjective_animal_value_for_player(
                        all_animals,
                        &animals,
                        animal,
                        *trade_count,
                    );

                    let my_value = Self::subjective_animal_value_for_player(
                        all_animals,
                        &owned_animals,
                        animal,
                        *trade_count,
                    );

                    let total_value_change = my_value + opponent_value;
                    sorted_trades.insert((
                        FloatOrd(total_value_change),
                        id.clone(),
                        animal.clone(),
                        *trade_count,
                    ));
                }
            }
        }

        sorted_trades
    }

    pub fn get_bill_combination(&self, amount: Value) -> Vec<Money> {
        let possible_combinations = self.wallet.propose_bill_combinations(amount, false);
        match possible_combinations.first() {
            Some((_, bill_combination)) => bill_combination.clone(),
            None => Vec::new(),
        }
    }

    pub fn get_highest_bid(bids: &Vec<(PlayerId, Bidding)>) -> Option<(&PlayerId, &Value)> {
        match bids.iter().max_by_key(|(_, bid)| bid) {
            Some((player_id, Bidding::Bid(value))) => Some((player_id, value)),
            Some(&(_, Bidding::Pass())) => None,
            None => None,
        }
    }

    pub fn trade_helper(&mut self) -> Option<(bool, InitialTrade)> {
        let value_save_to_spend = (self.wallet.total_money() as f32 * self.aggressiveness) as usize;

        let mut sub_optimal_trade: Option<InitialTrade> = None;

        let sorted_trades = self.sorted_opponent_trades();
        for (_, opponent_id, animal, animal_count) in sorted_trades.iter() {
            let (_, opponent_value) = self.opponents.get(opponent_id).unwrap();

            if opponent_value.max <= value_save_to_spend {
                let bill_combination = self.get_bill_combination(opponent_value.max);
                let save_trade = InitialTrade {
                    opponent: opponent_id.clone(),
                    animal: *animal,
                    animal_count: *animal_count,
                    amount: bill_combination.clone(),
                };
                return Some((true, save_trade));
            }

            if sub_optimal_trade.is_none() && opponent_value.max <= self.wallet.total_money() {
                let bill_combination = self.get_bill_combination(opponent_value.value_at_step(0.5));
                sub_optimal_trade = Some(InitialTrade {
                    opponent: opponent_id.clone(),
                    animal: *animal,
                    animal_count: *animal_count,
                    amount: bill_combination.clone(),
                });
            }
        }

        if sub_optimal_trade.is_some() {
            return Some((false, sub_optimal_trade.unwrap()));
        }

        match sorted_trades.last() {
            Some(_) => {
                let (_, opponent_id, animal, animal_count) = sorted_trades.last().unwrap();

                let (money, _) = self
                    .wallet
                    .bank_notes()
                    .last_key_value()
                    .unwrap_or((&0, &0));

                Some((
                    false,
                    InitialTrade {
                        opponent: opponent_id.clone(),
                        animal: animal.clone(),
                        animal_count: *animal_count,
                        amount: vec![money.clone()],
                    },
                ))
            }
            None => None,
        }
    }

    pub fn average_subj_value_over_last(&self, count: usize) -> f32 {
        let mut averaged_subj_values = 0.0;
        if self.previous_subjective_values.len() > count {
            let average_from_index = self.previous_subjective_values.len() - count;
            averaged_subj_values = self.previous_subjective_values[average_from_index..]
                .iter()
                .sum::<f32>()
                / count as f32;
        } else if self.previous_subjective_values.len() > 0 {
            averaged_subj_values = self.previous_subjective_values.iter().sum::<f32>()
                / self.previous_subjective_values.len() as f32;
        }

        averaged_subj_values
    }
}
