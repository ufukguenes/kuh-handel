use serde::{Deserialize, Serialize};

use crate::messages::actions::FromActionMessage;

use crate::messages::message_protocol::StateMessage;
use crate::model::animals::{Animal, AnimalSet};

use crate::model::game_errors::GameError;
use crate::model::money::wallet::Wallet;
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use std::collections::HashMap;
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
    owned_animals: HashMap<Animal, usize>,
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
            owned_animals: HashMap::new(),
            player_actions,
        }
    }

    pub fn id(&self) -> &PlayerId {
        &self.id
    }

    pub fn can_trade(&self) -> (Option<(Animal, usize)>, bool) {
        for (&animal, &animal_count) in self.owned_animals.iter() {
            if animal_count
                < self
                    .game_stack
                    .iter()
                    .find(|set| set.animal() == &animal)
                    .map(|set| set.occurrences())
                    .unwrap()
            {
                return (Some((animal, animal_count)), true);
            }
        }
        (None, false)
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

    pub fn owned_animals(&self) -> &HashMap<Animal, usize> {
        &self.owned_animals
    }

    pub fn add_animals(&mut self, animal: &Animal, count: usize) {
        self.owned_animals
            .entry(*animal)
            .and_modify(|current| *current += count)
            .or_insert(count);
    }

    pub fn remove_animals(&mut self, animal: &Animal, count: usize) -> Result<(), GameError> {
        let current_count = self.owned_animals.get_mut(animal);
        match current_count {
            Some(current_count) => {
                if *current_count - count > 0 {
                    *current_count -= count;
                } else if *current_count - count > 0 {
                    self.owned_animals.remove(animal);
                } else {
                    return Result::Err(GameError::AnimalsNotAvailable);
                }
            }
            None => return Result::Err(GameError::AnimalsNotAvailable),
        }

        Ok(())
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
