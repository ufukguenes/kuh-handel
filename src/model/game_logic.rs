use crate::model::animals::Animal;
use crate::model::animals::AnimalSet;
use crate::model::money::money::Money;
use crate::model::money::wallet::Wallet;
use crate::model::player::base_player::Player;
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use crate::model::player::player_group::PlayerGroup;
use crate::player_actions::actions::{AuctionAction, AuctionValue, PlayerTurnDecision};
use crate::player_actions::game_updates::{AuctionRound, GameUpdate};
use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;

use std::cell::RefCell;
use std::collections::HashMap;

use std::fmt;
use std::fmt::Display;
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

    pub fn get_player_by_id(&self, id: String) -> Option<Rc<RefCell<Player>>> {
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
        // ToDo: replace the dummy
        player.consume_animal(animal);
    }

    fn trade(
        &mut self,
        challenger: Rc<RefCell<Player>>,
        opponent: Rc<RefCell<Player>>,
        amount: Vec<Money>,
        animal: Animal,
    ) {
        // Trigger the trade between challenger and opponent
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
                PlayerTurnDecision::Trade {
                    opponent,
                    animal,
                    amount,
                } => {
                    let opponent = Rc::clone(&self.players.borrow().get_by_id(&opponent).unwrap());
                    self.trade(player, opponent, amount, animal);
                }
            };
            current_player_idx = (current_player_idx + 1) % players.borrow().len();
            println!("");

            // ToDo: a lot of stuff to do here
        }
    }

    fn trading_phase(&mut self) {}
}
