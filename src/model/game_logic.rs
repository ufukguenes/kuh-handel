use crate::model::animals::Animal;
use crate::model::animals::AnimalSet;
use crate::model::money::wallet::Wallet;
use crate::model::player::base_player::{FirstPhaseAction, Player, TradeAmount};
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use crate::model::player::player_group::PlayerGroup;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use tokio::sync::MutexGuard;

use std::collections::HashMap;
use tokio::sync::Mutex;

use std::fmt;
use std::fmt::Display;

use std::sync::Arc;

pub struct Game<T>
where
    T: PlayerActions,
{
    players: Arc<Mutex<PlayerGroup<T>>>,
    game_stack: Vec<Arc<Animal>>,
    animal_usage: HashMap<Arc<Animal>, Arc<AnimalSet>>,
    animal_sets: Vec<Arc<AnimalSet>>,
    num_players: usize,
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

impl<T> Game<T>
where
    T: PlayerActions,
{
    pub fn new(players: PlayerGroup<T>, animal_sets: Vec<Arc<AnimalSet>>, seed: u64) -> Self {
        let mut animal_usage: HashMap<Arc<Animal>, Arc<AnimalSet>> = HashMap::new();
        let mut game_stack: Vec<Arc<Animal>> = Vec::new();
        let num_players = players.len();

        for set in animal_sets.iter() {
            for animal in set.animals() {
                animal_usage.insert(Arc::clone(animal), Arc::clone(set));
                game_stack.push(Arc::clone(animal));
            }
        }

        game_stack.shuffle(&mut ChaCha8Rng::seed_from_u64(seed));

        Game {
            players: Arc::new(Mutex::new(players)),
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

    pub async fn num_players(&mut self) -> usize {
        self.num_players = self.players.lock().await.len();
        self.num_players
    }

    pub async fn get_all_ids(&self) -> Vec<String> {
        let mut all_ids = Vec::new();
        for player in self.players.lock().await.iter() {
            all_ids.push(player.lock().await.id().to_string());
        }
        return all_ids;
    }

    pub async fn get_player_by_id(&self, id: String) -> Option<Arc<Mutex<Player<T>>>> {
        for player in self.players.lock().await.iter() {
            if player.lock().await.id() == id {
                return Some(Arc::clone(player));
            }
        }

        return None;
    }

    pub async fn get_player_for_current_turn(&self) -> Arc<Mutex<Player<T>>> {
        self.players.lock().await.get(0).unwrap() // todo
    }

    pub fn remove_player(&mut self, id: String) {}

    pub fn play_one_round(&mut self) {}

    fn auction(&mut self, player: &mut Player<T>, animal: &Animal) {
        // ToDo: replace the dummy
        player.consume_animal(animal);
    }

    fn trade(
        &mut self,
        challenger: MutexGuard<Player<T>>,
        opponent: Arc<Mutex<Player<T>>>,
        amount: TradeAmount,
        animal: Animal,
    ) {
        // Trigger the trade between challenger and opponent
    }

    async fn draw_phase(&mut self) {
        let mut current_player_idx = 0usize;
        // get player order and iterate over them
        // draw a card and trigger the auction
        //   in the auction ask each player to bid, and provide the current transaction state = tuple of player and his/her current/highest bid
        //
        while !self.game_stack.is_empty() {
            println!("--- New turn ---");
            let players = self.players.clone();
            let mut players = players.lock().await;
            let player = players.get(current_player_idx).unwrap();
            let mut player = player.lock().await;
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
                    let opponent = players.get_by_id_mut(&opponent).await.unwrap();
                    self.trade(player, opponent, amount, animal);
                }
            };
            current_player_idx = (current_player_idx + 1) % players.len();
            println!("");

            // ToDo: a lot of stuff to do here
        }
    }

    fn trading_phase(&mut self) {}
}
