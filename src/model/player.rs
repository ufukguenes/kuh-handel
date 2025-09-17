use crate::model::animals::Animal;
use crate::model::game_logic::Game;

use super::money::Money;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

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
}

pub enum AuctionValue {
    Bidding(Money),
    Pass,
}

// ToDo: we need the number of cards to be visible for the opponent -> based on the amount he has to decide whether to take the deal or place a counter bid
pub struct TradeAmount {
    amount: Vec<Money>,
}

pub struct PlayerId {
    name: String,
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
    id: String,
    wallet: Wallet,
    owned_animals: Vec<Animal>, // ToDo: maybe all this construct "stall"
    pub player_actions: T,
}

impl<T> Player<T>
where
    T: PlayerActions,
{
    pub fn new(id: String, wallet: Wallet, player_actions: T) -> Self {
        Player {
            id: id,
            wallet: wallet,
            owned_animals: Vec::new(),
            player_actions,
        }
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
    players: Vec<Player<T>>,
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
                .map(|(id, player_action)| Player::new(id.clone(), wallet.clone(), player_action))
                .collect(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Player<T>> {
        self.players.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Player<T>> {
        self.players.iter_mut()
    }

    pub fn get(&self, index: usize) -> Result<&Player<T>, GameError> {
        self.players.get(index).ok_or(GameError::PlayerNotFound)
    }

    pub fn get_by_id_mut(&mut self, id: &PlayerId) -> Result<&mut Player<T>, GameError> {
        self.players
            .iter_mut()
            .find(|p| &p.id == &id.name)
            .ok_or(GameError::PlayerNotFound)
    }

    pub fn get_mut(&mut self, index: usize) -> Result<&mut Player<T>, GameError> {
        self.players.get_mut(index).ok_or(GameError::PlayerNotFound)
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
