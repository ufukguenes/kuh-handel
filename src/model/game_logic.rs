use crate::model::animals::Animal;
use crate::model::animals::AnimalSet;
use crate::model::player::FirstPhaseAction;
use crate::model::player::Player;
use crate::model::player::PlayerActions;
use crate::model::player::PlayerGroup;
use crate::model::player::TradeAmount;
use std::collections::HashMap;

pub struct Game<T>
where
    T: PlayerActions,
{
    players: PlayerGroup<T>,
    game_stack: Vec<Animal>,
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
    pub fn new(players: PlayerGroup<T>, animal_sets: Vec<AnimalSet>) -> Self {
        let animal_usage: HashMap<Animal, AnimalSet> = HashMap::new();
        let mut game_stack = Vec::<Animal>::new();
        animal_sets
            .iter()
            .for_each(|set| game_stack.append(&mut set.animals()));
        todo!();
        Game {
            players: players,
            game_stack: game_stack,
            animal_usage: animal_usage,
        }
    }

    pub fn play(&mut self) -> GameResult {
        self.draw_phase();
        self.trading_phase();

        Ok(())
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
