use crate::model::animals::Animal;
use crate::model::money::wallet::Wallet;
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use crate::player_actions::actions::{AuctionAction, AuctionValue, PlayerTurnDecision};
use crate::player_actions::game_updates::{AuctionRound, GameUpdate};
use std::fmt;
use std::fmt::Display;
#[derive(PartialEq, Eq)]
pub struct PlayerId {
    name: String,
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
    pub player_actions: Box<dyn PlayerActions>,
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
        println!("Player {} consumes animal {}", self.id(), animal,);
        self.owned_animals.push(*animal);
        self.wallet.withdraw(animal.value()).unwrap();
    }

    pub fn id(&self) -> &str {
        &self.id.name
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl PlayerActions for Player {
    fn provide_bidding(&mut self, state: AuctionRound) -> AuctionValue {
        self.player_actions.provide_bidding(state)
    }

    fn draw_or_trade(&mut self) -> PlayerTurnDecision {
        self.player_actions.draw_or_trade()
    }

    fn buy_or_sell(&mut self, state: AuctionRound) -> AuctionAction {
        self.player_actions.buy_or_sell(state)
    }

    fn receive_game_update(&mut self, update: GameUpdate) {
        self.player_actions.receive_game_update(update)
    }
}
