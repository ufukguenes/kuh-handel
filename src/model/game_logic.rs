use crate::model::animals::Animal;
use crate::model::animals::AnimalSet;
use crate::model::player::FirstPhaseAction;
use crate::model::player::Player;
use crate::model::player::PlayerActions;
use crate::model::player::PlayerGroup;
use crate::model::player::TradeAmount;
use std::collections::HashMap;

use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

pub struct Game<T>
where
    T: PlayerActions,
{
    players: PlayerGroup<T>,
    game_stack: Vec<Rc<Animal>>,
    animal_usage: HashMap<Rc<Animal>, Rc<AnimalSet>>,
    animal_sets: Vec<Rc<AnimalSet>>,
}

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
            self.players.len(),
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
    pub fn new(players: PlayerGroup<T>, animal_sets: Vec<Rc<AnimalSet>>) -> Self {
        let mut animal_usage: HashMap<Rc<Animal>, Rc<AnimalSet>> = HashMap::new();
        let mut game_stack: Vec<Rc<Animal>> = Vec::new();

        for set in animal_sets.iter() {
            for animal in set.animals() {
                animal_usage.insert(Rc::clone(animal), Rc::clone(set));
                game_stack.push(Rc::clone(animal));
            }
        }

        Game {
            players: players,
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

    pub fn num_players(self) -> usize {
        self.players.len()
    }

    pub fn get_all_ids(&self) -> Vec<String> {
        self.players
            .iter()
            .map(|p| p.get_string_id().clone())
            .collect()
    }

    pub fn get_player_by_id(&self, id: String) -> &Player<T> {
        self.players
            .iter()
            .find(|p| p.get_string_id() == id)
            .ok_or(err) // todo
    }

    pub fn get_player_for_current_turn(&self) -> &Player<T> {
        self.players.get(0).unwrap() // todo
    }

    fn auction(&mut self, player: &mut Player<T>, animal: Animal) {
        // Do the auction with the animal
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
            let mut player = self.players.get_mut(current_player_idx).unwrap();
            // ToDo: we need to work with RefCell and Rc to avoid borrow issues
            // match player.draw_or_trade() {
            //     FirstPhaseAction::Draw => self.auction(player, self.game_stack.pop().unwrap()),
            //     FirstPhaseAction::Trade {
            //         opponent,
            //         animal,
            //         amount,
            //     } => self.trade(
            //         player,
            //         self.players.get_by_id_mut(&opponent).unwrap(),
            //         amount,
            //         animal,
            //     ),
            // }
            // current_player_idx += 1;
        }
    }

    fn trading_phase(&mut self) {}
}
