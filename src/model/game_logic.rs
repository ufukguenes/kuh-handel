use crate::messages::actions::AuctionDecision;
use crate::messages::actions::Bidding;
use crate::messages::actions::InitialTrade;
use crate::messages::actions::PlayerTurnDecision;
use crate::messages::actions::TradeOffer;
use crate::messages::actions::TradeOpponentDecision;
use crate::messages::game_updates::AnimalTradeCount;
use crate::messages::game_updates::AuctionRound;
use crate::messages::game_updates::GameUpdate;
use crate::messages::game_updates::MoneyTransfer;
use crate::model::animals::Animal;
use crate::model::animals::AnimalSet;
use crate::model::money::money::Money;
use crate::model::money::value::Value;
use crate::model::player::base_player::Player;
use crate::model::player::base_player::PlayerId;
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use crate::model::player::player_group::PlayerGroup;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;

use std::cell::RefCell;
use std::collections::HashMap;

use std::fmt;
use std::fmt::Display;
use std::os::unix::process;
use std::rc::Rc;

pub struct Game {
    players: Rc<RefCell<PlayerGroup>>,
    game_stack: Vec<Rc<Animal>>,
    animal_usage: HashMap<Rc<Animal>, Rc<AnimalSet>>,
    animal_sets: Vec<Rc<AnimalSet>>,
    num_players: usize,
}

#[derive(Debug)]
pub enum GameError {
    InvalidAction,
    InvalidState,
}

type GameResult<T = ()> = Result<T, GameError>;

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
    pub fn new(players: PlayerGroup, animal_sets: Vec<Rc<AnimalSet>>, seed: u64) -> Self {
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

        Game {
            players: Rc::new(RefCell::new(players)),
            game_stack: game_stack,
            animal_usage: animal_usage,
            animal_sets: animal_sets,
            num_players: num_players,
        }
    }

    pub fn play(&mut self) -> GameResult {
        self.draw_phase();
        self.trading_phase();

        Ok(())
    }

    pub fn num_players(&mut self) -> usize {
        self.num_players = self.players.borrow().len();
        self.num_players
    }

    pub fn get_all_ids(&self) -> Vec<String> {
        let mut all_ids = Vec::new();
        for player in self.players.borrow().iter() {
            all_ids.push(player.borrow().id().to_string());
        }
        return all_ids;
    }

    pub fn get_player_by_id(&self, id: &PlayerId) -> Option<Rc<RefCell<Player>>> {
        for player in self.players.borrow().iter() {
            if player.borrow().id() == id {
                return Some(Rc::clone(player));
            }
        }

        return None;
    }

    pub fn get_player_for_current_turn(&self) -> Rc<RefCell<Player>> {
        self.players.borrow().get(0).unwrap() // todo
    }

    pub fn remove_player(&mut self, id: String) {}

    pub fn play_one_round(&mut self) {}

    fn auction(&mut self, player: &mut Player, animal: &Animal) {
        let players = Rc::clone(&self.players);
        let auction_players = players.borrow().get_auction_players(player.id());

        let host_id = player.id();
        let mut bids = Vec::<(PlayerId, Bidding)>::new();
        let mut pass_count = 0usize;

        for bidder in auction_players.iter() {
            let bidding = bidder.borrow_mut().provide_bidding(AuctionRound {
                animal: animal.clone(),
                host: host_id.clone(),
                bids: bids.clone(),
            });
            bids.push((bidder.borrow().id().clone(), bidding.clone()));

            println!(
                "Player {} bids {:?} in auction for animal {}",
                bidder.borrow().id(),
                bidding,
                animal
            );

            if let Bidding::Pass = bidding {
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
        let player_decision = player.buy_or_sell(final_auction_round.clone());

        // ToDo: if no one has bidden, what will happen?

        let (max_bidder_id, max_bid) = bids.iter().max_by_key(|(_, bid)| bid).unwrap();
        let auction_winner = auction_players
            .iter()
            .find(|p| p.borrow().id() == max_bidder_id)
            .unwrap();
        let mut auction_winner = auction_winner.borrow_mut();

        let max_bid = match max_bid {
            Bidding::Pass => Value::new(0),
            Bidding::Bid(v) => v.get_value(),
        };

        let (sender, receiver) = match player_decision {
            AuctionDecision::Buy => {
                println!("Player {} buys animal {}", player.id(), animal);
                player.consume_animal(animal);

                (player, &mut *auction_winner)
            }
            AuctionDecision::Sell => {
                println!("Player {} sells animal {}", player.id(), animal);
                auction_winner.consume_animal(animal);

                (&mut *auction_winner, player)
            }
        };

        let auction_result = Self::process_auction_and_get_game_update(
            sender,
            receiver,
            max_bid,
            final_auction_round,
        );

        for p in players.borrow().iter() {
            p.borrow_mut().receive_game_update(auction_result.clone());
        }
    }

    fn process_auction_and_get_game_update(
        sender: &mut Player,
        receiver: &mut Player,
        max_bid: Value,
        final_auction_round: AuctionRound,
    ) -> GameUpdate {
        let bid_money = sender.send_money_to_player(receiver.id(), max_bid);
        receiver.receive_from_player(sender.id(), bid_money.clone());

        let update = GameUpdate::Auction {
            rounds: final_auction_round,
            transfer: MoneyTransfer {
                from: sender.id().clone(),
                to: receiver.id().clone(),
                card_amount: bid_money.len(),
                min_value: max_bid, // ToDo: calculate the min value
            },
        };

        todo!("implement the money transfer");
        update
    }

    fn offer_trade_to_opponent(
        &mut self,
        challenger: &mut Player,
        opponent: &mut Player,
        amount: Vec<Money>,
        animal: Animal,
        animal_count: AnimalTradeCount,
    ) {
        let offer = TradeOffer {
            challenger: challenger.id().clone(),
            animal,
            animal_count,
            challenger_card_offer: amount.len(),
        };
        let decision = opponent.respond_to_trade(offer);
        match decision {
            TradeOpponentDecision::Accept => {
                println!("Trade accepted by {}", opponent.id());
            }
            TradeOpponentDecision::CounterOffer { amount } => {
                println!(
                    "Trade countered by {} with amount {:?}",
                    opponent.id(),
                    amount
                );
            }
        }
        todo!("calculate the winner of the trade and exchange the animals and money accordingly");
    }

    fn player_must_trade(&mut self, player: &mut Player) {
        let trade = player.trade();
        let opponent = self.players.borrow().get_by_id(&trade.opponent).unwrap();

        self.offer_trade_to_opponent(
            player,
            &mut *opponent.borrow_mut(),
            trade.amount,
            trade.animal,
            trade.animal_count,
        );
    }

    fn draw_phase(&mut self) {
        let mut current_player_idx = 0usize;
        // get player order and iterate over them
        // draw a card and trigger the auction
        //   in the auction ask each player to bid, and provide the current transaction state = tuple of player and his/her current/highest bid
        //
        while !self.game_stack.is_empty() {
            println!("--- New turn ---");
            let players = Rc::clone(&self.players);
            let player = Rc::clone(&players.borrow().get(current_player_idx).unwrap());

            let action = player.borrow_mut().draw_or_trade();
            match action {
                PlayerTurnDecision::Draw => {
                    let card = self.game_stack.pop().unwrap();
                    println!("Player {} drew card: {}", player.borrow().id(), card);
                    self.auction(&mut *player.borrow_mut(), &card)
                }
                PlayerTurnDecision::Trade(InitialTrade {
                    opponent,
                    animal,
                    animal_count,
                    amount,
                }) => {
                    let opponent = Rc::clone(&self.players.borrow().get_by_id(&opponent).unwrap());
                    self.offer_trade_to_opponent(
                        &mut *player.borrow_mut(),
                        &mut *opponent.borrow_mut(),
                        amount,
                        animal,
                        animal_count,
                    );
                }
            };
            current_player_idx = (current_player_idx + 1) % players.borrow().len();
            println!("");

            // ToDo: a lot of stuff to do here
        }
    }

    fn trading_phase(&mut self) {
        let players = Rc::clone(&self.players);
        for player in players.borrow().iter().cycle() {
            let mut current_player = player.borrow_mut();

            if current_player.can_trade() {
                self.player_must_trade(&mut *current_player);
            }
        }
    }
}
