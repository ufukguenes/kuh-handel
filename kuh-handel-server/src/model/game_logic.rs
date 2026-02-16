use kuh_handel_lib::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney,
    TradeOpponentDecision,
};
use kuh_handel_lib::messages::game_updates::{
    AuctionKind, AuctionRound, GameUpdate, MoneyTrade, MoneyTransfer, Points, TradeOffer,
};

use kuh_handel_lib::animals::{Animal, AnimalSet};
use kuh_handel_lib::messages::message_protocol::StateMessage;
use kuh_handel_lib::player::{
    base_player::{Player, PlayerId},
    wallet::Wallet,
};
use kuh_handel_lib::{Money, Value};
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

use crate::game_error::GameError;

use crate::server_side_player::supervised_player::SupervisedPlayer;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use tracing::error;

use std::collections::BTreeMap;

use std::fmt;
use std::fmt::Display;
use std::ops::Not;
use std::sync::Arc;

pub struct Game {
    players: Vec<Arc<Mutex<SupervisedPlayer>>>,
    game_stack: Vec<Arc<Animal>>,
    animal_usage: BTreeMap<Arc<Animal>, Arc<Mutex<AnimalSet>>>,
    animal_sets: Vec<Arc<Mutex<AnimalSet>>>,
    num_players: usize,
    num_bidding_rounds: usize,
    num_trading_rounds: usize,
    num_drawing_rounds: usize,
    start_wallet: Wallet,
}

impl Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "num_players: {}\nsize_game_stack: {}\ngame_stack: \n",
            self.num_players,
            self.game_stack.len()
        )?;

        for (i, animal) in self.game_stack.clone().iter().enumerate() {
            write!(f, "     {}: {}\n", i, animal)?;
        }
        write!(f, " \nnum_animal_sets: {} \n", self.animal_sets.len())?;

        for (i, set) in self.animal_sets.iter().enumerate() {
            write!(f, "     {}: {}\n", i, set.blocking_lock())?;
        }
        write!(f, "")
    }
}

impl Game {
    pub fn new(
        players: Vec<Player>,
        start_wallet: Wallet,
        animal_sets: Vec<Arc<Mutex<AnimalSet>>>,
        seed: u64,
        num_bidding_rounds: usize,
        num_trading_rounds: usize,
        num_drawing_rounds: usize,
    ) -> Self {
        let mut animal_usage: BTreeMap<Arc<Animal>, Arc<Mutex<AnimalSet>>> = BTreeMap::new();
        let mut game_stack: Vec<Arc<Animal>> = Vec::new();
        let num_players = players.len();

        for set in animal_sets.iter() {
            let binding = set.blocking_lock();
            for animal in binding.animals() {
                animal_usage.insert(Arc::clone(animal), Arc::clone(set));
                game_stack.push(Arc::clone(animal));
            }
        }
        game_stack.shuffle(&mut ChaCha8Rng::seed_from_u64(seed));

        let players: Vec<Arc<Mutex<Player>>> = players
            .into_iter()
            .map(|p| Arc::new(Mutex::new(p)))
            .collect();

        let mut supervised_players = Vec::new();
        for player in players.iter() {
            let current_id = player.blocking_lock().id().clone();
            let opponents: Vec<Arc<Mutex<Player>>> = players
                .iter()
                .filter(|p| *p.blocking_lock().id() != current_id)
                .cloned()
                .collect();
            let new_supervised_player = SupervisedPlayer::new(player.clone(), opponents);
            supervised_players.push(Arc::new(Mutex::new(new_supervised_player)));
        }
        let players_in_turn_order = supervised_players
            .iter()
            .map(|ref_player| ref_player.blocking_lock().id().clone())
            .collect();

        let animals = animal_sets
            .iter()
            .map(|rc_animal| rc_animal.blocking_lock().clone())
            .collect::<Vec<AnimalSet>>();

        let wallet = start_wallet.clone();

        let update = GameUpdate::Start {
            wallet: wallet,
            players_in_turn_order: players_in_turn_order,
            animals: animals,
        };
        Self::update_multiple_players(&supervised_players, update);

        Game {
            players: supervised_players,
            game_stack: game_stack,
            animal_usage: animal_usage,
            animal_sets: animal_sets,
            num_players: num_players,
            start_wallet: start_wallet,
            num_bidding_rounds: num_bidding_rounds,
            num_trading_rounds: num_trading_rounds,
            num_drawing_rounds: num_drawing_rounds,
        }
    }

    pub fn play(&mut self) -> Result<Vec<(PlayerId, Points)>, GameError> {
        self.draw_phase();
        self.trading_phase();

        if !self.validate_players_animals() {
            // print!("gl | game failed animal check at the end");
            return Err(GameError::InvalidAnimalsAtEnd);
        };

        if !self.validate_players_money() {
            // print!("gl | game failed money check at the end");
            return Err(GameError::InvalidMoneyAtEnd);
        };

        let mut ranking: Vec<(PlayerId, Points)> = self
            .players
            .iter()
            .map(|p| {
                let binding = p.blocking_lock();
                (binding.id(), binding.calculate_points())
            })
            .collect();

        ranking.sort_by(|(_, points_a), (_, points_b)| points_b.cmp(points_a));

        let update = GameUpdate::End {
            ranking: ranking.clone(),
            illegal_moves_made: Vec::default(),
        };
        Self::update_multiple_players(&self.players, update);
        Ok(ranking)
    }

    pub fn validate_players_money(&self) -> bool {
        let mut all_money_from_players: BTreeMap<Money, usize> = BTreeMap::new();
        for player in self.players.iter() {
            let binding = player.blocking_lock().clone_wallet();
            let current_bank_notes = binding.bank_notes();

            for (money, count) in current_bank_notes.into_iter() {
                all_money_from_players
                    .entry(money.clone())
                    .and_modify(|total| *total += count)
                    .or_insert(*count);
            }
        }

        all_money_from_players = all_money_from_players
            .into_iter()
            .map(|(money, count)| (money, count / self.num_players))
            .collect();

        if all_money_from_players == *self.start_wallet.bank_notes() {
            true
        } else {
            error!("gl | validation: all player money doesn't add up to total money generated");
            error!("gl | validation: \t {:?}", all_money_from_players);
            error!("gl | validation: \t {:?}", *self.start_wallet.bank_notes());

            false
        }
    }

    pub fn validate_players_animals(&self) -> bool {
        let mut beginning_animals = self.animal_sets.clone();
        for (i, player) in self.players.iter().enumerate() {
            let current_animals = player.blocking_lock().clone_owned_animals();
            // check if animal occurs the right amount of times
            for (animal, count) in current_animals.into_iter() {
                let pos = beginning_animals.iter().position(|set| {
                    let binding = set.blocking_lock();
                    *binding.animal() == animal
                });
                match pos {
                    Some(pos) => {
                        let current_set = beginning_animals.remove(pos);
                        if current_set.blocking_lock().occurrences() != count {
                            {
                                error!(
                                    "gl | validation: wrong occurrences {} instead of {}",
                                    count,
                                    current_set.blocking_lock().occurrences()
                                );

                                for player in self.players.iter() {
                                    error!(
                                        "gl | validation: \t {:?}",
                                        player.blocking_lock().clone_owned_animals()
                                    );
                                }

                                for player in self.players.iter() {
                                    error!(
                                        "gl | validation: \t {:?}",
                                        player.blocking_lock().clone_wallet()
                                    );
                                }

                                return false;
                            };
                        }
                    }
                    None => {
                        error!(
                            "gl | validation: player {} has animal {} that does not exist",
                            player.blocking_lock().id(),
                            animal
                        );
                        return false;
                    }
                }
            }

            // check that no two players share the same animal
            for other in self.players[i + 1..].iter() {
                let binding = other.blocking_lock();
                let others_animals = binding.clone_owned_animals();

                if others_animals.is_empty() {
                    return true;
                }

                let has_shared = others_animals.keys().all(|animal| {
                    player
                        .blocking_lock()
                        .clone_owned_animals()
                        .contains_key(animal)
                });
                if has_shared {
                    // println!(
                    //    "{} and {} share animals",
                    //    other.blocking_lock().id(),
                    //    player.blocking_lock().id()
                    //);
                    error!(
                        "gl | validation: {} and {} share animals",
                        other.blocking_lock().id(),
                        player.blocking_lock().id()
                    );

                    for player in self.players.iter() {
                        error!(
                            "gl | validation: \t {:?}",
                            player.blocking_lock().clone_owned_animals()
                        );
                    }

                    for player in self.players.iter() {
                        error!(
                            "gl | validation: \t {:?}",
                            player.blocking_lock().clone_wallet()
                        );
                    }
                    return false;
                }
            }
        }

        if beginning_animals.len() > 0 {
            error!(
                "gl | validation: animals that existed are now missing {:?}",
                beginning_animals
            );
            return false;
        }

        true
    }

    pub fn num_players(&mut self) -> usize {
        self.num_players = self.players.len();
        self.num_players
    }

    pub fn get_all_ids(&self) -> Vec<String> {
        let mut all_ids = Vec::new();
        for player in self.players.iter() {
            all_ids.push(player.blocking_lock().id().to_string());
        }
        return all_ids;
    }

    fn auction(&mut self, player: Arc<Mutex<SupervisedPlayer>>, animal: &Arc<Animal>) {
        let host_id = player.blocking_lock().id().clone();

        let auction_players = self.get_players_excluding(vec![host_id.clone()]);
        // print!("gl | \t host {}, auction_player: ", host_id);
        for p in auction_players.clone() {
            // print!("{}, ", p.blocking_lock().id().name);
        }
        // println!();

        let mut bids = Vec::<(PlayerId, Bidding)>::new();

        for _ in 0..self.num_bidding_rounds {
            let mut pass_count = 0usize;
            for bidder in auction_players.iter() {
                let auction_round = AuctionRound {
                    animal: Arc::clone(&animal),
                    host: host_id.clone(),
                    bids: bids.clone(),
                };
                let state_msg = StateMessage::ProvideBidding {
                    state: auction_round,
                };
                let player_decision: Bidding =
                    bidder.blocking_lock().map_to_action_inner(state_msg);

                bids.push((bidder.blocking_lock().id().clone(), player_decision.clone()));

                // println!(
                //    "gl | \t\t Player {} bids {:?} in auction for animal {}",
                //    bidder.blocking_lock().id(),
                //    player_decision,
                //    animal
                // );

                if let Bidding::Pass() = player_decision {
                    pass_count += 1;
                }
            }

            if pass_count == auction_players.len() {
                break;
            }
        }

        match bids.iter().max_by_key(|(_, bid)| bid) {
            Some((max_bidder_id, max_bid)) => {
                let auction_winner = auction_players
                    .iter()
                    .find(|p| p.blocking_lock().id() == *max_bidder_id)
                    .unwrap(); // if there is a bid there will also always be a player who made that bid
                let auction_winner = auction_winner;

                let max_bid = match max_bid {
                    Bidding::Pass() => 0,
                    Bidding::Bid(value) => *value,
                };

                if max_bid == 0 {
                    let update: GameUpdate = GameUpdate::Auction(AuctionKind::NoBiddings {
                        host_id: host_id.clone(),
                        animal: **animal,
                    });

                    Self::update_multiple_players(&self.players, update);
                    return;
                }

                let final_auction_round = AuctionRound {
                    host: host_id.clone(),
                    animal: animal.clone(),
                    bids: bids.clone(),
                };

                let state_msg = StateMessage::BuyOrSell {
                    state: final_auction_round.clone(),
                };
                let player_decision: AuctionDecision =
                    player.blocking_lock().map_to_action_inner(state_msg);

                let (sender, receiver) = match player_decision {
                    AuctionDecision::Buy() => {
                        // println!(
                        //    "gl | \t Host {} buys animal {} from {} with bid {}",
                        //    host_id, animal, max_bidder_id, max_bid
                        //);

                        (player, Arc::clone(auction_winner))
                    }
                    AuctionDecision::Sell() => {
                        // println!(
                        //    "gl | \t Host {} sells animal {} to {} with bid {}",
                        //    host_id, animal, max_bidder_id, max_bid
                        //);

                        (Arc::clone(auction_winner), player)
                    }
                };

                self.process_auction(sender, receiver, max_bid, final_auction_round);
            }
            None => {
                let update = GameUpdate::Auction(AuctionKind::NoBiddings {
                    host_id: host_id,
                    animal: **animal,
                });

                Self::update_multiple_players(&self.players, update);
            }
        };
    }

    fn process_auction(
        &mut self,
        sender: Arc<Mutex<SupervisedPlayer>>,
        receiver: Arc<Mutex<SupervisedPlayer>>,
        max_bid: Value,
        final_auction_round: AuctionRound,
    ) {
        let state_msg = StateMessage::SendMoney {
            player_id: receiver.blocking_lock().id().clone(),
            amount: max_bid,
        };

        let player_decision: SendMoney = sender.blocking_lock().map_to_action_inner(state_msg);
        match player_decision {
            SendMoney::WasBluff() => {
                let player_id = sender.blocking_lock().id();
                let player_wallet = sender.blocking_lock().clone_wallet();
                let update = GameUpdate::ExposePlayer {
                    player: player_id,
                    wallet: player_wallet,
                };
                // limit for the player is enforced in supervised_player until auction is over, hence this will execute at most "number of players" many times

                Self::update_multiple_players(&self.players, update);
                let host = self.get_by_id(final_auction_round.host).unwrap(); // player always exists
                // println!(
                //    "gl | \t player {} bluffed, exposed value {}",
                //    sender.blocking_lock().id(),
                //    sender.blocking_lock().clone_wallet().total_money(),
                // );

                self.auction(host, &final_auction_round.animal);
            }
            SendMoney::Amount(amount) => {
                let sender_id = sender.blocking_lock().id().clone();

                let receiver_id = receiver.blocking_lock().id().clone();

                let rounds = final_auction_round.clone();

                let public_kind = MoneyTransfer::Public {
                    card_amount: amount.len(),
                    min_value: max_bid,
                };

                // println!(
                //    "gl | \t player {} sends {} bills to {}",
                //    sender.blocking_lock().id(),
                //    amount.len(),
                //    receiver.blocking_lock().id().clone()
                //);

                let update = |transfer_kind| {
                    GameUpdate::Auction(AuctionKind::NormalAuction {
                        from: sender_id.clone(),
                        to: receiver_id.clone(),
                        rounds: rounds.clone(),
                        money_transfer: transfer_kind,
                    })
                };

                let private_kind = MoneyTransfer::Private { amount: amount };

                self.public_private_update(
                    sender,
                    receiver,
                    update(public_kind),
                    update(private_kind),
                );
            }
        }
    }

    fn update_multiple_players(players: &Vec<Arc<Mutex<SupervisedPlayer>>>, update: GameUpdate) {
        let mut handles = Vec::new();

        for player_arc in players {
            let player_arc = Arc::clone(player_arc);
            let update_clone = match update.clone() {
                GameUpdate::End {
                    ranking,
                    illegal_moves_made: _,
                } => {
                    let personalized_illegal_moves =
                        player_arc.blocking_lock().illegal_moves_made.clone();
                    GameUpdate::End {
                        ranking,
                        illegal_moves_made: personalized_illegal_moves,
                    }
                }
                _ => update.clone(),
            };

            let handle = std::thread::spawn(move || {
                let mut binding = player_arc.blocking_lock();

                let _: NoAction = binding.map_to_action_inner(StateMessage::GameUpdate {
                    update: update_clone,
                });
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }
    }

    fn public_private_update(
        &self,
        player_a: Arc<Mutex<SupervisedPlayer>>,
        player_b: Arc<Mutex<SupervisedPlayer>>,
        public_update: GameUpdate,
        private_update: GameUpdate,
    ) {
        let player_a_id = player_a.blocking_lock().id().clone();
        let player_b_id = player_b.blocking_lock().id().clone();
        let other_player = self.get_players_excluding(vec![player_a_id, player_b_id]);
        Self::update_multiple_players(&other_player, public_update);
        Self::update_multiple_players(&vec![player_a, player_b], private_update);
    }

    fn offer_trade_to_opponent(
        &self,
        challenger: Arc<Mutex<SupervisedPlayer>>,
        opponent: Arc<Mutex<SupervisedPlayer>>,
        amount: Vec<Money>,
        animal: Animal,
        animal_count: usize,
    ) {
        let challenger_total_value: usize = amount.iter().map(|money| money).sum();
        let challenger_card_count = amount.len();
        let challenger_offer_vec = amount;
        let offer: TradeOffer = TradeOffer {
            challenger: challenger.blocking_lock().id().clone(),
            animal: animal,
            animal_count: animal_count.clone(),
            challenger_card_offer: challenger_card_count,
        };

        let state_msg = StateMessage::RespondToTrade { offer: offer };
        let player_decision: TradeOpponentDecision =
            opponent.blocking_lock().map_to_action_inner(state_msg);

        let (opponent_total_value, opponent_card_count, opponent_offer_vec) = match player_decision
        {
            TradeOpponentDecision::Accept() => {
                // println!("gl | \t Trade accepted by {}", opponent.blocking_lock().id());
                (0, None, None)
            }
            TradeOpponentDecision::CounterOffer(amount) => {
                // println!(
                //    "gl | \t Trade countered by {} with amount {}",
                //    opponent.blocking_lock().id(),
                //    amount.iter().map(|m| m.as_usize()).sum::<usize>(),
                //);
                (
                    amount.iter().map(|money| money).sum(),
                    Some(amount.len()),
                    Some(amount),
                )
            }
        };

        let winner_id = if challenger_total_value >= opponent_total_value {
            challenger.blocking_lock().id().clone()
        } else {
            opponent.blocking_lock().id().clone()
        };

        let challenger_id = challenger.blocking_lock().id().clone();
        let opponent_id = opponent.blocking_lock().id().clone();
        let update = |trade_kind| GameUpdate::Trade {
            challenger: challenger_id,
            opponent: opponent_id,
            animal: animal,
            animal_count: animal_count,
            receiver: winner_id,
            money_trade: trade_kind,
        };

        let public_kind = MoneyTrade::Public {
            challenger_card_offer: challenger_card_count,
            opponent_card_offer: opponent_card_count,
        };
        let private_kind = MoneyTrade::Private {
            challenger_card_offer: challenger_offer_vec,
            opponent_card_offer: opponent_offer_vec,
        };

        self.public_private_update(
            challenger,
            opponent,
            update.clone()(public_kind),
            update(private_kind),
        );
    }

    fn player_must_trade(&self, player: Arc<Mutex<SupervisedPlayer>>) {
        // println!("gl | {} must trade", player.blocking_lock().id());

        let state_msg = StateMessage::Trade();
        let player_decision: InitialTrade = player.blocking_lock().map_to_action_inner(state_msg);

        let opponent = self.get_by_id(player_decision.opponent);
        match opponent {
            Ok(opponent) => {
                let opponent_count = opponent
                    .blocking_lock()
                    .player
                    .blocking_lock()
                    .owned_animals()
                    .get(&player_decision.animal)
                    .unwrap_or(&0)
                    .clone();

                // println!(
                //   "gl | \t {} trades {}-{} for {}, against {} who has {} many",
                //   player.blocking_lock().id(),
                ////   player_decision.animal_count,
                //   player_decision.animal,
                //   player_decision
                //       .amount
                //      .iter()
                //      .map(|m| m.as_usize())
                //      .sum::<usize>(),
                //    player_decision.opponent,
                //   opponent_count
                //);

                self.offer_trade_to_opponent(
                    player,
                    opponent,
                    player_decision.amount,
                    player_decision.animal,
                    player_decision.animal_count,
                )
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    fn process_card_inflation(&mut self, card: &Animal) {
        let mut animal_set = self.animal_usage.get(card).unwrap().blocking_lock(); // must not fail as all animals of the game are in the mapping

        let inflation = animal_set.get_next_inflation();
        if inflation > 0 {
            self.start_wallet.add_money(inflation);
            let update = GameUpdate::Inflation(inflation);
            Self::update_multiple_players(&self.players, update);
        }
        animal_set.increase_draw_count();
    }

    fn draw_phase(&mut self) {
        let mut current_player_idx = 0usize;
        // get player order and iterate over them
        // draw a card and trigger the auction
        //   in the auction ask each player to bid, and provide the current transaction state = tuple of player and his/her current/highest bid
        //

        let mut current_draw_round_count = 0usize;
        while !self.game_stack.is_empty() && current_draw_round_count < self.num_drawing_rounds {
            current_draw_round_count += 1;

            // println!("gl | --- New turn ---");

            let player: Arc<Mutex<SupervisedPlayer>> =
                Arc::clone(self.players.get(current_player_idx).unwrap());

            let state_msg = StateMessage::DrawOrTrade();
            let player_decision: PlayerTurnDecision =
                player.blocking_lock().map_to_action_inner(state_msg);

            match player_decision {
                PlayerTurnDecision::Draw() => {
                    let card = self.game_stack.pop().unwrap();

                    self.process_card_inflation(&card);

                    // println!(
                    //    "gl | \t Player {} drew card: {}",
                    //    player.blocking_lock().id(),
                    //    card
                    // );
                    self.auction(player, &card)
                }
                PlayerTurnDecision::Trade(InitialTrade {
                    opponent,
                    animal,
                    animal_count,
                    amount,
                }) => {
                    let opponent = self.get_by_id(opponent);
                    match opponent {
                        Ok(opponent) => {
                            let opponent_count = opponent
                                .blocking_lock()
                                .player
                                .blocking_lock()
                                .owned_animals()
                                .get(&animal)
                                .unwrap_or(&0)
                                .clone();

                            // println!(
                            //   "gl | \t {} trades {}-{} for {}, against {} who has {} many",
                            //   player.blocking_lock().id(),
                            //    animal_count,
                            //   animal,
                            //   amount.iter().map(|m| m.as_usize()).sum::<usize>(),
                            //   opponent.blocking_lock().id(),
                            //   opponent_count
                            //);
                            self.offer_trade_to_opponent(
                                player,
                                opponent,
                                amount,
                                animal,
                                animal_count,
                            )
                        }
                        Err(e) => panic!("{:?}", e),
                    }
                }
            };
            current_player_idx = (current_player_idx + 1) % self.players.len();
        }
    }

    fn trading_phase(&mut self) {
        let mut skip_players: Vec<PlayerId> = Vec::new();

        for idx in 0..self.num_trading_rounds {
            for player in self.players.iter() {
                if skip_players.len() == self.players.len() {
                    // println!("gl | game ended as no player can trade anymore");
                    break;
                }

                let player_id = player.blocking_lock().id().clone();
                if !skip_players.contains(&player_id) {
                    let can_trade = player.blocking_lock().can_trade().is_some();
                    if can_trade {
                        self.player_must_trade(Arc::clone(player));
                    } else {
                        // println!(
                        //    "gl | player will be {} skipped in trading",
                        //    player.blocking_lock().id()
                        //);
                        skip_players.push(player_id);
                    }
                }
            }
        }
    }

    pub fn get_players_excluding(
        &self,
        excluding: Vec<PlayerId>,
    ) -> Vec<Arc<Mutex<SupervisedPlayer>>> {
        self.players
            .iter()
            .filter(|p| excluding.contains(&&p.blocking_lock().id()).not())
            .cloned()
            .collect()
    }

    pub fn get_by_id(
        &self,
        player_id: PlayerId,
    ) -> Result<Arc<Mutex<SupervisedPlayer>>, GameError> {
        let player = self
            .players
            .iter()
            .find(|player| player.blocking_lock().id() == *player_id);

        match player {
            Some(player) => Ok(Arc::clone(player)),
            None => Err(GameError::PlayerNotFound),
        }
    }
}
