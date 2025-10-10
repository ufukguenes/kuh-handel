use serde::{Deserialize, Serialize};

use crate::messages::actions::{FromActionMessage, InitialTrade};

use crate::messages::game_updates::Points;
use crate::messages::message_protocol::StateMessage;
use crate::model::animals::{Animal, AnimalSet};

use crate::model::game_errors::GameError;
use crate::model::money::wallet::Wallet;
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerId {
    pub name: String,
}

impl PlayerId {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct Player {
    id: PlayerId,
    wallet: Wallet,
    owned_animals: BTreeMap<Animal, usize>,
    game_stack: Vec<Rc<AnimalSet>>,
    player_actions: Box<dyn PlayerActions>,
}

impl Player {
    pub fn new(
        id: String,
        wallet: Wallet,
        game_stack: Vec<Rc<AnimalSet>>,
        player_actions: Box<dyn PlayerActions>,
    ) -> Self {
        Player {
            id: PlayerId { name: id },
            wallet: wallet,
            game_stack: game_stack,
            owned_animals: BTreeMap::new(),
            player_actions,
        }
    }

    pub fn id(&self) -> &PlayerId {
        &self.id
    }

    pub fn calculate_points(&self) -> Points {
        let mut animal_sum: Points = 0;
        for (animal, _) in self.owned_animals.iter() {
            animal_sum += animal.value().value();
        }
        return animal_sum * self.owned_animals.len();
    }

    pub fn can_trade(&self, opponents: &Vec<Rc<RefCell<Player>>>) -> Option<InitialTrade> {
        for opponent in opponents.iter() {
            let possible_trade = self.can_trade_against(Rc::clone(opponent));
            match possible_trade {
                Some(trade) => {
                    return Some(trade);
                }
                None => {
                    continue;
                }
            }
        }
        None
    }

    pub fn can_trade_animal(
        &self,
        animal: &Animal,
        opponents: &Vec<Rc<RefCell<Player>>>,
    ) -> Option<InitialTrade> {
        if let Some(&animal_count) = self.owned_animals.get(animal) {
            for opponent in opponents.iter() {
                if let Some(&opponent_animal_count) = opponent.borrow().owned_animals().get(animal)
                {
                    let max_trade_count = std::cmp::min(animal_count, opponent_animal_count);

                    return Some(InitialTrade {
                        opponent: opponent.borrow().id().clone(),
                        animal: animal.clone(),
                        animal_count: max_trade_count,
                        amount: Vec::new(),
                    });
                }
            }
        }
        None
    }

    pub fn can_trade_against(&self, opponent: Rc<RefCell<Player>>) -> Option<InitialTrade> {
        for (&animal, &animal_count) in self.owned_animals.iter() {
            if animal_count
                < self
                    .game_stack
                    .iter()
                    .find(|set| set.animal() == &animal)
                    .map(|set| set.occurrences())
                    .unwrap()
            {
                let binding = opponent.borrow();
                let opponent_animals = binding.owned_animals();
                if let Some(&opponent_animal_count) = opponent_animals.get(&animal) {
                    let max_trade_count = std::cmp::min(animal_count, opponent_animal_count);
                    return Some(InitialTrade {
                        opponent: opponent.borrow().id().clone(),
                        animal: animal,
                        animal_count: max_trade_count,
                        amount: Vec::new(),
                    });
                }
            }
        }

        None
    }

    pub fn map_to_action_inner<T: FromActionMessage>(&mut self, state_msg: StateMessage) -> T {
        let action_msg = self.player_actions.map_to_action(state_msg);
        T::extract(action_msg)
    }

    pub fn player_actions(&mut self) -> &mut dyn PlayerActions {
        self.player_actions.as_mut()
    }

    pub fn wallet(&self) -> &Wallet {
        &self.wallet
    }

    pub fn wallet_mut(&mut self) -> &mut Wallet {
        &mut self.wallet
    }

    pub fn owned_animals(&self) -> &BTreeMap<Animal, usize> {
        &self.owned_animals
    }

    pub fn add_animals(&mut self, animal: &Animal, count: usize) {
        self.owned_animals
            .entry(*animal)
            .and_modify(|current| *current += count)
            .or_insert(count);
    }

    pub fn remove_animals(&mut self, animal: &Animal, count: usize) -> Result<(), GameError> {
        let backup_animals = self.owned_animals.clone();
        let current_count = self.owned_animals.get_mut(animal);
        match current_count {
            Some(current_count) => {
                let res: isize = *current_count as isize - count as isize;
                if res > 0 {
                    *current_count -= count;
                } else if *current_count == 0 || res == 0 {
                    self.owned_animals.remove(animal);
                } else {
                    self.owned_animals = backup_animals;
                    return Result::Err(GameError::AnimalsNotAvailable);
                }
            }
            None => {
                self.owned_animals = backup_animals;
                return Result::Err(GameError::AnimalsNotAvailable);
            }
        }

        Ok(())
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
