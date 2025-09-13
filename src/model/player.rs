use crate::model::animals::Animal;

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

pub struct TradeAmount {
    amount: Vec<Money>,
}

pub struct PlayerId {
    name: String,
}

pub enum FirstPhaseAction {
    Draw,
    Trade {
        player: PlayerId,
        amout: TradeAmount,
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
    fn provide_bidding(state: AuctionState) -> AuctionValue;
    fn draw_or_trade() -> FirstPhaseAction;
    fn buy_or_sell(state: AuctionState) -> AuctionAction;
}

pub struct RandomPlayerActions {}

impl PlayerActions for RandomPlayerActions {
    fn provide_bidding(state: AuctionState) -> AuctionValue {
        AuctionValue::Pass
    }

    fn draw_or_trade() -> FirstPhaseAction {
        FirstPhaseAction::Draw
    }

    fn buy_or_sell(state: AuctionState) -> AuctionAction {
        AuctionAction::Buy
    }
}

pub struct Player<T: PlayerActions> {
    id: String,
    wallet: Wallet,
    owned_animals: Vec<Animal>, // ToDo: maybe all this construct "stall"
    player_actions: T,
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
}
