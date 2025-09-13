use crate::model::animals::Animal;
use crate::model::animals::AnimalSet;
use crate::model::player::PlayerActions;
use crate::model::player::PlayerGroup;
use std::collections::HashMap;

pub enum DrawingAction {
    Bid,
}

pub struct Game<T>
where
    T: PlayerActions,
{
    players: PlayerGroup<T>,
    game_stack: Vec<AnimalSet>,
    animal_usage: HashMap<Animal, AnimalSet>,
}

pub enum GameError {
    InvalidAction,
    InvalidState,
}

type GameResult<T = ()> = Result<T, GameError>;

impl<T> Game<T>
where
    T: PlayerActions,
{
    pub fn new(players: PlayerGroup<T>, game_stack: Vec<AnimalSet>) -> Self {
        let animal_usage: HashMap<Animal, AnimalSet> = HashMap::new();
        todo!();
        Game {
            players: players,
            game_stack: game_stack,
            animal_usage: animal_usage,
        }
    }

    pub fn main_loop(&mut self) -> GameResult {
        self.draw_phase();
        self.trading_phase();

        Ok(())
    }

    fn draw_phase(&mut self) {
        // get player order and iterate over them
        // draw a card and trigger the auction
        //   in the auction ask each player to bid, and provide the current transaction state = tuple of player and his/her current/highest bid
        //
    }

    fn trading_phase(&mut self) {}
}
