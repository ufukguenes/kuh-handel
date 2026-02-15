use crate::messages::message_protocol::ActionMessage;
use crate::{Money, Value, animals};
use crate::{animals::Animal, player::base_player::PlayerId};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::error;
pub trait FromActionMessage: Sized {
    fn extract(action: ActionMessage) -> Option<Self>;
}

/// Action of the bot that is used for conformation if the server sends a game update that does not require the bot to do anything
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum NoAction {
    Ok(),
}
impl FromActionMessage for NoAction {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::NoAction { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::NoAction");
                None
            }
        }
    }
}

/// Action to decide if a bot, whose turn it currently is, wants to draw a new card or trade a card it already owns
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PlayerTurnDecision {
    Draw(),
    Trade(InitialTrade),
}

impl FromActionMessage for PlayerTurnDecision {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::PlayerTurnDecision { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::PlayerTurnDecision");
                None
            }
        }
    }
}

#[pymethods]
impl PlayerTurnDecision {
    #[getter]
    fn initial_trade(&self) -> PyResult<Option<InitialTrade>> {
        match self {
            PlayerTurnDecision::Trade(it) => Ok(Some(it.clone())),
            _ => Ok(None),
        }
    }
}

/// Action to specify the a trade offer
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct InitialTrade {
    /// The player id of the opponent with which to trade
    #[pyo3(get)]
    pub opponent: PlayerId,

    /// The animal that is offered for the trade
    #[pyo3(get)]
    pub animal: Animal,

    /// The number of animals to trade. The opponent also needs to own this number of the specified animal.
    #[pyo3(get)]
    pub animal_count: usize,

    /// Action to describe a trade that is initialized by the called bot
    #[pyo3(get)]
    pub amount: Vec<Money>,
}

#[pymethods]
impl InitialTrade {
    #[new]
    fn new(opponent: PlayerId, animal: Animal, animal_count: usize, amount: Vec<Money>) -> Self {
        InitialTrade {
            opponent,
            animal,
            animal_count,
            amount,
        }
    }
}

impl FromActionMessage for InitialTrade {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::InitialTrade { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::InitialTrade");
                None
            }
        }
    }
}

/// Action for the bot which hosted a auction after drawing a card.
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AuctionDecision {
    /// The host wants to buy the animal itself, by paying out the highest bidder
    Buy(),

    /// The host wants to sell the animal to the highest bidder
    Sell(),
}
impl FromActionMessage for AuctionDecision {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::AuctionDecision { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::AuctionDecision");
                None
            }
        }
    }
}

/// Action to decide to either accept a trade offer from another player or make a counter offer, with a combination of cards/ bills
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum TradeOpponentDecision {
    Accept(),
    CounterOffer(Vec<Money>),
}

impl FromActionMessage for TradeOpponentDecision {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::TradeOpponentDecision { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::TradeOpponentDecision");
                None
            }
        }
    }
}

#[pymethods]
impl TradeOpponentDecision {
    #[getter]
    fn money_list(&self) -> PyResult<Option<Vec<Money>>> {
        match self {
            TradeOpponentDecision::CounterOffer(m) => Ok(Some(m.clone())),
            _ => Ok(None),
        }
    }
}

/// Action to specify what combination of cards/ bills is send to another player to fulfill a requested minimum amount, or to acknowledge that it was a bluff  
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SendMoney {
    WasBluff(),
    Amount(Vec<Money>),
}

#[pymethods]
impl SendMoney {
    #[getter]
    fn money_list(&self) -> PyResult<Option<Vec<Money>>> {
        match self {
            SendMoney::Amount(money_list) => Ok(Some(money_list.clone())),
            _ => Ok(None),
        }
    }
}

impl FromActionMessage for SendMoney {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::SendMoney { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::SendMoney");
                None
            }
        }
    }
}

/// Action to specify a single bid for the current animal that is auctioned
#[pyclass(unsendable)]
#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub enum Bidding {
    /// Do not place a bid, if every other player also bids the auction is over, if not a player is allowed to join in the next bidding round again
    Pass(),

    /// Places a bid with the specified value
    Bid(Value),
}

#[pymethods]
impl Bidding {
    #[getter]
    fn money_list(&self) -> PyResult<Option<Value>> {
        match self {
            Bidding::Bid(val) => Ok(Some(val.clone())),
            _ => Ok(None),
        }
    }
}

impl PartialEq for Bidding {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Bidding::Pass(), Bidding::Pass()) => true,
            (Bidding::Pass(), _) => false,
            (_, Bidding::Pass()) => false,
            (Bidding::Bid(a), Bidding::Bid(b)) => a == b,
        }
    }
}

impl Ord for Bidding {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Bidding::Pass(), Bidding::Pass()) => std::cmp::Ordering::Equal,
            (Bidding::Pass(), _) => std::cmp::Ordering::Less,
            (_, Bidding::Pass()) => std::cmp::Ordering::Greater,
            (Bidding::Bid(a), Bidding::Bid(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for Bidding {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl FromActionMessage for Bidding {
    fn extract(action: ActionMessage) -> Option<Self> {
        match action {
            ActionMessage::Bidding { decision } => Some(decision),
            _ => {
                error!("Expected ActionMessage::Bidding");
                None
            }
        }
    }
}
