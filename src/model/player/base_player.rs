use crate::model::animals::Animal;
use crate::model::game_logic::Game;
use crate::model::money::value::Value;
use crate::model::money::wallet::Wallet;

use crate::model::money::money::Money;
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts::E;
use std::fmt;
use std::fmt::Display;
use std::io::empty;
use std::rc::Rc;

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

pub enum FirstPhaseAction {
    Draw,
    Trade {
        opponent: PlayerId,
        animal: Animal,
        amount: TradeAmount,
    },
}

pub enum AuctionAction {
    Buy,
    Sell,
}

pub struct AuctionState {
    animal: Animal,
    mapping: HashMap<PlayerId, AuctionValue>,
}

pub enum AuctionValue {
    Bidding(Money),
    Pass,
}

// ToDo: we need the number of cards to be visible for the opponent -> based on the amount he has to decide whether to take the deal or place a counter bid
pub struct TradeAmount {
    amount: Vec<Money>,
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
    fn provide_bidding(&mut self, state: AuctionState) -> AuctionValue {
        self.player_actions.provide_bidding(state)
    }

    fn draw_or_trade(&mut self) -> FirstPhaseAction {
        self.player_actions.draw_or_trade()
    }

    fn buy_or_sell(&mut self, state: AuctionState) -> AuctionAction {
        self.player_actions.buy_or_sell(state)
    }
}
