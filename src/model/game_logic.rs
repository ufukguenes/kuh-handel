use crate::model::animals::Animal;
use crate::model::animals::AnimalSet;
use crate::model::player::base_player::FirstPhaseAction;
use crate::model::player::base_player::Player;
use crate::model::player::base_player::TradeAmount;
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use crate::model::player::player_group::PlayerGroup;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use std::cell::RefCell;
use std::collections::HashMap;

use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

pub struct Game<T>
where
    T: PlayerActions,
{
    players: Rc<RefCell<PlayerGroup<T>>>,
    game_stack: Vec<Rc<Animal>>,
    animal_usage: HashMap<Rc<Animal>, Rc<AnimalSet>>,
    animal_sets: Vec<Rc<AnimalSet>>,
}

#[derive(Debug)]
pub enum GameError {
    InvalidAction,
    InvalidState,
}

type GameResult<T = ()> = Result<T, GameError>;

impl<T> Display for Game<T>
where
    T: PlayerActions,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "num_players: {}\nsize_game_stack: {}\ngame_stack: \n",
            self.players.borrow().len(),
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

impl<T> Game<T>
where
    T: PlayerActions,
{
    pub fn new(players: PlayerGroup<T>, animal_sets: Vec<Rc<AnimalSet>>, seed: u64) -> Self {
        let mut animal_usage: HashMap<Rc<Animal>, Rc<AnimalSet>> = HashMap::new();
        let mut game_stack: Vec<Rc<Animal>> = Vec::new();

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
        }
    }

    pub fn play(&mut self) -> GameResult {
        self.draw_phase();
        self.trading_phase();

        Ok(())
    }

    pub fn num_players(&self) -> usize {
        self.players.borrow().len()
    }

    pub fn get_all_ids(&self) -> Vec<String> {
        self.players
            .borrow()
            .iter()
            .map(|p| p.borrow().id().to_string())
            .collect()
    }

    pub fn get_player_by_id(&self, id: String) -> Result<Rc<RefCell<Player<T>>>, &str> {
        self.players
            .borrow()
            .iter()
            .find(|p| p.borrow().id() == id)
            .map(|p| Rc::clone(p))
            .ok_or("err") // todo
    }

    pub fn get_player_for_current_turn(&self) -> Rc<RefCell<Player<T>>> {
        self.players.borrow().get(0).unwrap() // todo
    }

    fn auction(&mut self, player: &mut Player<T>, animal: &Animal) {
        // ToDo: replace the dummy
        player.consume_animal(animal);
    }

    fn trade(
        &mut self,
        challenger: &mut Player<T>,
        opponent: &mut Player<T>,
        amount: TradeAmount,
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
            let players = self.players.clone();
            let mut players = players.borrow_mut();
            let player = players.get(current_player_idx).unwrap();
            let mut player = player.borrow_mut();
            match player.draw_or_trade() {
                FirstPhaseAction::Draw => {
                    let card = self.game_stack.pop().unwrap();
                    println!("Player {} drew card: {}", player.id(), card);
                    self.auction(&mut *player, &card)
                }
                FirstPhaseAction::Trade {
                    opponent,
                    animal,
                    amount,
                } => {
                    let opponent = players.get_by_id_mut(&opponent).unwrap();
                    let mut opponent = opponent.borrow_mut();
                    self.trade(&mut *player, &mut *opponent, amount, animal);
                }
            };
            current_player_idx = (current_player_idx + 1) % players.len();
            println!("");

            // ToDo: a lot of stuff to do here
        }
    }

    fn trading_phase(&mut self) {}
}
