use crate::py_messages::py_actions::*;
use crate::py_messages::py_game_updates::*;
use crate::py_player::py_base_player::PlayerId;
use crate::Value;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pymodule]
pub fn message_protocol_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<ActionMessage>();
    m.add_class::<StateMessage>();

    Ok(())
}

#[pyclass]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ActionMessage {
    PlayerTurnDecision { decision: PlayerTurnDecision },
    InitialTrade { decision: InitialTrade },
    Bidding { decision: Bidding },
    AuctionDecision { decision: AuctionDecision },
    SendMoney { decision: SendMoney },
    TradeOpponentDecision { decision: TradeOpponentDecision },
    NoAction { decision: NoAction },
}

#[pyclass]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum StateMessage {
    DrawOrTrade(),
    Trade(),
    ProvideBidding { state: AuctionRound },
    BuyOrSell { state: AuctionRound },
    SendMoney { player_id: PlayerId, amount: Value },
    RespondToTrade { offer: TradeOffer },
    GameUpdate { update: GameUpdate },
}
