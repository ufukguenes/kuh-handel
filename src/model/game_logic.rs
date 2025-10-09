use crate::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction, PlayerTurnDecision, SendMoney, TradeOffer,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{
    AuctionKind, AuctionRound, GameUpdate, MoneyTrade, MoneyTransfer,
};

use crate::messages::message_protocol::StateMessage;
use crate::model::animals::{Animal, AnimalSet};
use crate::model::game_errors::GameError;
use crate::model::money::money::Money;
use crate::model::money::value::Value;
use crate::model::money::wallet::Wallet;
use crate::model::player::base_player::{Player, PlayerId};

use crate::model::player::supervised_player::SupervisedPlayer;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;

use std::cell::RefCell;
use std::collections::HashMap;

use std::fmt;
use std::fmt::Display;
use std::ops::Not;
use std::rc::Rc;

pub struct Game {
    players: Vec<Rc<RefCell<SupervisedPlayer>>>,
    game_stack: Vec<Rc<Animal>>,
    animal_usage: HashMap<Rc<Animal>, Rc<AnimalSet>>,
    animal_sets: Vec<Rc<AnimalSet>>,
    num_players: usize,
}

// todo inflation
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
            write!(f, "     {}: {}\n", i, set)?;
        }
        write!(f, "")
    }
}

impl Game {
    pub fn new(
        players: Vec<Player>,
        start_wallet: Wallet,
        animal_sets: Vec<Rc<AnimalSet>>,
        seed: u64,
    ) -> Self {
        let mut animal_usage: HashMap<Rc<Animal>, Rc<AnimalSet>> = HashMap::new();
        let mut game_stack: Vec<Rc<Animal>> = Vec::new();
        let num_players = players.len();

        for set in animal_sets.iter() {
            for animal in set.animals() {
                animal_usage.insert(Rc::clone(animal), Rc::clone(set));
                game_stack.push(Rc::clone(animal));
            }
        }

        game_stack.shuffle(&mut ChaCha8Rng::seed_from_u64(seed));

        let players: Vec<Rc<RefCell<Player>>> = players
            .into_iter()
            .map(|p| Rc::new(RefCell::new(p)))
            .collect();

        let mut supervised_players = Vec::new();
        for player in players.iter() {
            let opponents: Vec<Rc<RefCell<Player>>> = players
                .iter()
                .filter(|p| p.borrow().id() != player.borrow().id())
                .cloned()
                .collect();
            let new_supervised_player = SupervisedPlayer::new(player.clone(), opponents);
            supervised_players.push(Rc::new(RefCell::new(new_supervised_player)));
        }

        let update = GameUpdate::Start {
            wallet: start_wallet.clone(),
            players_in_turn_order: supervised_players
                .iter()
                .map(|ref_player| ref_player.borrow().id().clone())
                .collect(),
            animals: animal_sets
                .clone()
                .into_iter()
                .map(|rc_animal| (*rc_animal).clone())
                .collect(),
        };

        for player in supervised_players.iter() {
            let _: NoAction = player
                .borrow_mut()
                .map_to_action_inner(StateMessage::GameUpdate {
                    update: update.clone(),
                });
        }

        Game {
            players: supervised_players,
            game_stack: game_stack,
            animal_usage: animal_usage,
            animal_sets: animal_sets,
            num_players: num_players,
        }
    }

    pub fn play(&mut self) -> Result<(), GameError> {
        self.draw_phase();
        self.trading_phase();

        Ok(())

        /// todo check if game state is valid: 
        /// there are no more or less animals, 
        /// there is no more or less money, 
        /// everyone holds 4 (or num occurences) of each of their deck
    }

    pub fn num_players(&mut self) -> usize {
        self.num_players = self.players.len();
        self.num_players
    }

    pub fn get_all_ids(&self) -> Vec<String> {
        let mut all_ids = Vec::new();
        for player in self.players.iter() {
            all_ids.push(player.borrow().id().to_string());
        }
        return all_ids;
    }

    pub fn remove_player(&mut self, id: String) {}

    fn auction(&mut self, player: Rc<RefCell<SupervisedPlayer>>, animal: &Rc<Animal>) {
        let host_id = player.borrow().id().clone();

        let auction_players = self.get_players_excluding(vec![&host_id]);
        print!("gl | \t host {}, auction_player: ", host_id);
        for p in auction_players.clone() {
            print!("{}, ", p.borrow().id().name);
        }
        println!();

        let mut bids = Vec::<(PlayerId, Bidding)>::new();
        let mut pass_count = 0usize;

        for bidder in auction_players.iter() {
            // todo cycle but with max?
            let auction_round = AuctionRound {
                animal: Rc::clone(&animal),
                host: host_id.clone(),
                bids: bids.clone(),
            };
            let state_msg = StateMessage::ProvideBidding {
                state: auction_round,
            };
            let player_decision: Bidding = bidder.borrow_mut().map_to_action_inner(state_msg);

            bids.push((bidder.borrow().id().clone(), player_decision.clone()));

            println!(
                "gl | \t\t Player {} bids {:?} in auction for animal {}",
                bidder.borrow().id(),
                player_decision,
                animal
            );

            if let Bidding::Pass = player_decision {
                // todo should we be able to re join? and the pass count should be reset in this loop then, anyway currently just because a player passed doesnt exclude it from rebidding
                pass_count += 1;
            }

            if pass_count == auction_players.len() {
                break;
            }
        }

        let final_auction_round = AuctionRound {
            host: host_id.clone(),
            animal: animal.clone(),
            bids: bids.clone(),
        };
        let state_msg = StateMessage::BuyOrSell {
            state: final_auction_round.clone(),
        };
        let player_decision: AuctionDecision = player.borrow_mut().map_to_action_inner(state_msg);

        match bids.iter().max_by_key(|(_, bid)| bid) {
            Some((max_bidder_id, max_bid)) => {
                let auction_winner = auction_players
                    .iter()
                    .find(|p| p.borrow().id() == *max_bidder_id)
                    .unwrap();
                let auction_winner = auction_winner;

                let max_bid = match max_bid {
                    Bidding::Pass => Value::new(0),
                    Bidding::Bid(value) => *value,
                };

                let (sender, receiver) = match player_decision {
                    AuctionDecision::Buy => {
                        println!(
                            "gl | \t Host {} buys animal {} from {} with bid {}",
                            host_id, animal, max_bidder_id, max_bid
                        );

                        (player, Rc::clone(auction_winner))
                    }
                    AuctionDecision::Sell => {
                        println!(
                            "gl | \t Host {} sells animal {} to {} with bid {}",
                            host_id, animal, max_bidder_id, max_bid
                        );

                        (Rc::clone(auction_winner), player)
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
        sender: Rc<RefCell<SupervisedPlayer>>,
        receiver: Rc<RefCell<SupervisedPlayer>>,
        max_bid: Value,
        final_auction_round: AuctionRound,
    ) {
        let state_msg = StateMessage::SendMoney {
            player_id: receiver.borrow().id().clone(),
            amount: max_bid,
        };
        let player_decision: SendMoney = sender.borrow_mut().map_to_action_inner(state_msg);
        match player_decision {
            SendMoney::WasBluff => {
                let update = GameUpdate::ExposePlayer {
                    player: sender.borrow().id(),
                    wallet: sender.borrow().clone_wallet(),
                };
                // limit for the player is enforced in supervised_player until auction is over, hence this will execute at most "number of players" many times
                Self::update_multiple_players(&self.players, update);
                let host = self.get_by_id(&final_auction_round.host).unwrap();
                println!(
                    "gl | \t player {} bluffed, exposed value {}",
                    sender.borrow().id(),
                    sender
                        .borrow()
                        .clone_wallet()
                        .total_money()
                        .unwrap_or(Value::new(0)),
                );

                self.auction(host, &final_auction_round.animal);
            }
            SendMoney::Amount(amount) => {
                let sender_id = sender.borrow().id().clone();
                let receiver_id = receiver.borrow().id().clone();
                let rounds = final_auction_round.clone();
                let public_kind = MoneyTransfer::Public {
                    card_amount: amount.len(), // todo this is sometime 0, but that shouldnt be, why is the supervisior not working?
                    min_value: max_bid,        // ToDo: calculate the min value
                };

                println!(
                    "gl | \t player {} sends {} bills to {}",
                    sender.borrow().id(),
                    amount.len(),
                    receiver.borrow().id().clone()
                );

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

    fn update_multiple_players(players: &Vec<Rc<RefCell<SupervisedPlayer>>>, update: GameUpdate) {
        for other_player in players {
            let _: NoAction =
                other_player
                    .borrow_mut()
                    .map_to_action_inner(StateMessage::GameUpdate {
                        update: update.clone(),
                    });
        }
    }

    fn public_private_update(
        &self,
        player_a: Rc<RefCell<SupervisedPlayer>>,
        player_b: Rc<RefCell<SupervisedPlayer>>,
        public_update: GameUpdate,
        private_update: GameUpdate,
    ) {
        let other_player =
            self.get_players_excluding(vec![&player_a.borrow().id(), &player_b.borrow().id()]);

        Self::update_multiple_players(&other_player, public_update);

        let _: NoAction = player_a
            .borrow_mut()
            .map_to_action_inner(StateMessage::GameUpdate {
                update: private_update.clone(),
            });

        let _: NoAction = player_b
            .borrow_mut()
            .map_to_action_inner(StateMessage::GameUpdate {
                update: private_update,
            });
    }

    fn offer_trade_to_opponent(
        &self,
        challenger: Rc<RefCell<SupervisedPlayer>>,
        opponent: Rc<RefCell<SupervisedPlayer>>,
        amount: Vec<Money>,
        animal: Animal,
        animal_count: usize,
    ) {
        let challenger_total_value: usize = amount.iter().map(|money| money.as_usize()).sum();
        let challenger_card_count = amount.len();
        let challenger_offer_vec = amount;
        let offer: TradeOffer = TradeOffer {
            challenger: challenger.borrow().id().clone(),
            animal: animal,
            animal_count: animal_count.clone(),
            challenger_card_offer: challenger_card_count,
        };

        let state_msg = StateMessage::RespondToTrade { offer: offer };
        let player_decision: TradeOpponentDecision =
            opponent.borrow_mut().map_to_action_inner(state_msg);

        let (opponent_total_value, opponent_card_count, opponent_offer_vec) = match player_decision
        {
            TradeOpponentDecision::Accept => {
                println!("gl | \t Trade accepted by {}", opponent.borrow().id());
                (0, None, None)
            }
            TradeOpponentDecision::CounterOffer(amount) => {
                println!(
                    "gl | \t Trade countered by {} with amount {}",
                    opponent.borrow().id(),
                    amount.iter().map(|m| m.as_usize()).sum::<usize>(),
                );
                (
                    amount.iter().map(|money| money.as_usize()).sum(),
                    Some(amount.len()),
                    Some(amount),
                )
            }
        };

        let winner_id = if challenger_total_value >= opponent_total_value {
            challenger.borrow().id().clone()
        } else {
            opponent.borrow().id().clone()
        };

        let challenger_id = challenger.borrow().id().clone();
        let opponent_id = opponent.borrow().id().clone();
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

    fn player_must_trade(&self, player: Rc<RefCell<SupervisedPlayer>>) {
        println!("gl | {} must trade", player.borrow().id());

        let state_msg = StateMessage::Trade;
        let player_decision: InitialTrade = player.borrow_mut().map_to_action_inner(state_msg);

        let opponent = self.get_by_id(&player_decision.opponent);
        match opponent {
            Ok(opponent) => {
                let opponent_count = opponent
                    .borrow()
                    .player
                    .borrow()
                    .owned_animals()
                    .get(&player_decision.animal)
                    .unwrap_or(&0)
                    .clone();

                println!(
                    "gl | \t {} trades {}-{} for {}, against {} who has {} many",
                    player.borrow().id(),
                    player_decision.animal_count,
                    player_decision.animal,
                    player_decision
                        .amount
                        .iter()
                        .map(|m| m.as_usize())
                        .sum::<usize>(),
                    player_decision.opponent,
                    opponent_count
                );

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

    fn draw_phase(&mut self) {
        let mut current_player_idx = 0usize;
        // get player order and iterate over them
        // draw a card and trigger the auction
        //   in the auction ask each player to bid, and provide the current transaction state = tuple of player and his/her current/highest bid
        //
        while !self.game_stack.is_empty() {
            println!("gl | --- New turn ---");

            let player: Rc<RefCell<SupervisedPlayer>> =
                Rc::clone(self.players.get(current_player_idx).unwrap());

            let state_msg = StateMessage::DrawOrTrade;
            let player_decision: PlayerTurnDecision =
                player.borrow_mut().map_to_action_inner(state_msg);

            match player_decision {
                PlayerTurnDecision::Draw => {
                    let card = self.game_stack.pop().unwrap();
                    println!(
                        "gl | \t Player {} drew card: {}",
                        player.borrow().id(),
                        card
                    );
                    self.auction(player, &card)
                }
                PlayerTurnDecision::Trade(InitialTrade {
                    opponent,
                    animal,
                    animal_count,
                    amount,
                }) => {
                    let opponent = self.get_by_id(&opponent);
                    match opponent {
                        Ok(opponent) => {
                            let opponent_count = opponent
                                .borrow()
                                .player
                                .borrow()
                                .owned_animals()
                                .get(&animal)
                                .unwrap_or(&0)
                                .clone();

                            println!(
                                "gl | \t {} trades {}-{} for {}, against {} who has {} many",
                                player.borrow().id(),
                                animal_count,
                                animal,
                                amount.iter().map(|m| m.as_usize()).sum::<usize>(),
                                opponent.borrow().id(),
                                opponent_count
                            );
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
            println!("");

            // ToDo: a lot of stuff to do here
        }
    }

    fn trading_phase(&mut self) {
        let max_cycles = 1000; // todo find a better limit based on game stack
        let mut skip_players: Vec<PlayerId> = Vec::new();

        for (i, player) in self.players.iter().cycle().enumerate() {
            let current_cycle = i / self.players.len();
            if current_cycle >= max_cycles {
                println!(
                    "gl | game ended after maximum number of iterations in trading phase was reached "
                );
                break;
            }

            if skip_players.len() == self.players.len() {
                println!("gl | game ended as no player can trade anymore");
                break;
            }

            if !skip_players.contains(&player.borrow().id())
                && player.borrow().can_trade().is_some()
            {
                self.player_must_trade(Rc::clone(player));
            } else {
                skip_players.push(player.borrow().id().clone());
            }
        }
    }

    pub fn get_players_excluding(
        &self,
        excluding: Vec<&PlayerId>,
    ) -> Vec<Rc<RefCell<SupervisedPlayer>>> {
        self.players
            .iter()
            .filter(|p| excluding.contains(&&p.borrow().id()).not())
            .cloned()
            .collect()
    }

    pub fn get_by_id(
        &self,
        player_id: &PlayerId,
    ) -> Result<Rc<RefCell<SupervisedPlayer>>, GameError> {
        let player = self
            .players
            .iter()
            .find(|player| player.borrow().id() == *player_id);

        match player {
            Some(player) => Ok(Rc::clone(player)),
            None => Err(GameError::PlayerNotFound),
        }
    }
}
