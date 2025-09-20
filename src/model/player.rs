use crate::model::animals::Animal;
use crate::model::game_logic::Game;
use crate::model::money::Value;

use super::money::Money;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts::E;
use std::fmt;
use std::fmt::Display;
use std::io::empty;
use std::rc::Rc;

#[derive(Clone)]
pub struct Wallet {
    bank_notes: HashMap<Money, u32>,
}

impl Wallet {
    pub fn new(bank_notes: HashMap<Money, u32>) -> Self {
        Wallet {
            bank_notes: bank_notes,
        }
    }

    pub fn withdraw(&mut self, amount: Value) -> Result<(), GameError> {
        // ToDo: implement the actual version of withdraw (check money and maybe receive not a value but a handful of money)

        let key = self
            .bank_notes
            .iter()
            .find(|(key, _)| key.get_value() >= amount)
            .map(|(key, _)| *key);
        match key {
            Some(k) => {
                self.bank_notes
                    .entry(k)
                    .and_modify(|e| *e = e.checked_sub(1).or(Some(0)).unwrap());
            }
            None => (),
        };
        Ok(())
    }
}

pub enum AuctionValue {
    Bidding(Money),
    Pass,
}

// ToDo: we need the number of cards to be visible for the opponent -> based on the amount he has to decide whether to take the deal or place a counter bid
pub struct TradeAmount {
    amount: Vec<Money>,
}

#[derive(PartialEq, Eq)]
pub struct PlayerId {
    name: String,
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

pub trait PlayerActions {
    fn provide_bidding(&mut self, state: AuctionState) -> AuctionValue;
    fn draw_or_trade(&mut self) -> FirstPhaseAction;
    fn buy_or_sell(&mut self, state: AuctionState) -> AuctionAction;
}

pub struct RandomPlayerActions {}

impl PlayerActions for RandomPlayerActions {
    fn provide_bidding(&mut self, state: AuctionState) -> AuctionValue {
        AuctionValue::Pass
    }

    fn draw_or_trade(&mut self) -> FirstPhaseAction {
        FirstPhaseAction::Draw
    }

    fn buy_or_sell(&mut self, state: AuctionState) -> AuctionAction {
        AuctionAction::Buy
    }

    // ToDo: add the other actions -> the actual trade needs to be implemented (doing the attack as well as the counter bid)
}

pub struct Player<T: PlayerActions> {
    id: PlayerId,
    wallet: Wallet,
    owned_animals: Vec<Animal>,
    pub player_actions: T,
}

impl<T> Player<T>
where
    T: PlayerActions,
{
    pub fn new(id: String, wallet: Wallet, player_actions: T) -> Self {
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
        &self.id[..]
    }
}

impl<T> Display for Player<T>
where
    T: PlayerActions,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug)]
pub enum GameError {
    PlayerNotFound,
}

pub struct PlayerGroup<T: PlayerActions> {
    players: Vec<Rc<RefCell<Player<T>>>>,
}

impl<T> PlayerGroup<T>
where
    T: PlayerActions,
{
    pub fn new(player_ids: Vec<String>, player_actions: Vec<T>, wallet: Wallet) -> Self {
        PlayerGroup {
            players: player_ids
                .iter()
                .zip(player_actions)
                .map(|(id, player_action)| {
                    Rc::new(RefCell::new(Player::new(
                        id.clone(),
                        wallet.clone(),
                        player_action,
                    )))
                })
                .collect(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Rc<RefCell<Player<T>>>> {
        self.players.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Rc<RefCell<Player<T>>>> {
        self.players.iter_mut()
    }

    pub fn get(&self, index: usize) -> Result<Rc<RefCell<Player<T>>>, GameError> {
        self.players
            .get(index)
            .ok_or(GameError::PlayerNotFound)
            .cloned()
    }

    pub fn get_by_id_mut(&mut self, id: &PlayerId) -> Result<Rc<RefCell<Player<T>>>, GameError> {
        self.players
            .iter()
            .find(|p| p.borrow().id == id.name)
            .ok_or(GameError::PlayerNotFound)
            .cloned()
    }

    pub fn len(&self) -> usize {
        self.players.len()
    }
}

impl<T> PlayerActions for Player<T>
where
    T: PlayerActions,
{
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
