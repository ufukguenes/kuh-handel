use serde::{Deserialize, Serialize};

use crate::messages::actions::FromActionMessage;

use crate::messages::message_protocol::StateMessage;
use crate::model::animals::Animal;

use crate::model::money::wallet::Wallet;
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use std::fmt;
use std::fmt::Display;

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
    owned_animals: Vec<Animal>,
    player_actions: Box<dyn PlayerActions>,
}

impl Player {
    pub fn new(id: String, wallet: Wallet, player_actions: Box<dyn PlayerActions>) -> Self {
        Player {
            id: PlayerId { name: id },
            wallet: wallet,
            owned_animals: Vec::new(),
            player_actions,
        }
    }

    pub fn consume_animal(&mut self, animal: &Animal) {
        println!("bp | Player {} consumes animal {}", self.id(), animal,);
        self.owned_animals.push(*animal);

        todo!(
            "you don't necessarily pay the amount the animal is valued at, so why withdraw that?"
        );
        self.wallet.withdraw(animal.value()).unwrap();
    }

    pub fn id(&self) -> &PlayerId {
        &self.id
    }

    pub fn can_trade(&self) -> bool {
        todo!()
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

    pub fn owned_animals(&self) -> &Vec<Animal> {
        &self.owned_animals
    }

    pub fn count_animal(&self, animal_to_count: &Animal) -> usize {
        self.owned_animals()
            .iter()
            .filter(|animal| animal.value() == animal_to_count.value())
            .count()
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
