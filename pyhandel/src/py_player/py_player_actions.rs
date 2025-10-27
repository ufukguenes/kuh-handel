use crate::PlayerId;
use crate::Value;
use kuh_handel_lib::messages::{actions::*, game_updates::*, message_protocol::*};
use kuh_handel_lib::player::player_actions::PlayerActions as CorePlayerActions;
use pyo3::prelude::*;

#[pymodule]
pub fn player_actions_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<PlayerActions>();

    Ok(())
}

#[pyclass(unsendable, subclass, name = "PlayerActions")]
#[derive(Clone)]
pub struct PlayerActions;

#[pymethods]
impl PlayerActions {
    #[new]
    pub fn new() -> Self {
        PlayerActions {}
    }

    fn map_to_action(&mut self, state_msg: StateMessage) -> ActionMessage {
        match state_msg {
            StateMessage::DrawOrTrade() => ActionMessage::PlayerTurnDecision {
                decision: self._draw_or_trade(),
            },
            StateMessage::Trade() => ActionMessage::InitialTrade {
                decision: self._trade(),
            },
            StateMessage::ProvideBidding { state } => ActionMessage::Bidding {
                decision: self._provide_bidding(state),
            },
            StateMessage::BuyOrSell { state } => ActionMessage::AuctionDecision {
                decision: self._buy_or_sell(state),
            },
            StateMessage::SendMoney { player_id, amount } => ActionMessage::SendMoney {
                decision: self._send_money_to_player(player_id, amount),
            },
            StateMessage::RespondToTrade { offer } => ActionMessage::TradeOpponentDecision {
                decision: self._respond_to_trade(offer),
            },
            StateMessage::GameUpdate { update } => ActionMessage::NoAction {
                decision: self._receive_game_update(update),
            },
        }
    }

    pub fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        panic!("needs to be implemented by user in Python")
    }

    pub fn _trade(&mut self) -> InitialTrade {
        panic!("needs to be implemented by user in Python")
    }

    pub fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        panic!("needs to be implemented by user in Python")
    }

    pub fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        panic!("needs to be implemented by user in Python")
    }

    pub fn _send_money_to_player(&mut self, player: PlayerId, amount: Value) -> SendMoney {
        panic!("needs to be implemented by user in Python")
    }

    pub fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        panic!("needs to be implemented by user in Python")
    }

    pub fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        panic!("needs to be implemented by user in Python")
    }
}

pub struct RustPlayer {
    pub inner: Py<PlayerActions>,
}

impl RustPlayer {
    pub fn new(py_obj: Py<PlayerActions>) -> Self {
        Self { inner: py_obj }
    }
}

impl CorePlayerActions for RustPlayer {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        Python::with_gil(|py| {
            let result = self
                .inner
                .as_ref()
                .call_method0(py, "_draw_or_trade")
                .unwrap();
            result.extract::<PlayerTurnDecision>(py).unwrap()
        })
    }

    fn _trade(&mut self) -> InitialTrade {
        Python::with_gil(|py| {
            let result = self.inner.as_ref().call_method0(py, "_trade").unwrap();
            result.extract::<InitialTrade>(py).unwrap()
        })
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        Python::with_gil(|py| {
            let result = self
                .inner
                .as_ref()
                .call_method1(py, "_provide_bidding", (state,))
                .unwrap();
            result.extract::<Bidding>(py).unwrap()
        })
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        Python::with_gil(|py| {
            let result = self
                .inner
                .as_ref()
                .call_method1(py, "_buy_or_sell", (state,))
                .unwrap();
            result.extract::<AuctionDecision>(py).unwrap()
        })
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        Python::with_gil(|py| {
            let result = self
                .inner
                .as_ref()
                .call_method1(py, "_send_money_to_player", (player, amount))
                .unwrap();
            result.extract::<SendMoney>(py).unwrap()
        })
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        Python::with_gil(|py| {
            let result = self
                .inner
                .as_ref()
                .call_method1(py, "_respond_to_trade", (offer,))
                .unwrap();
            result.extract::<TradeOpponentDecision>(py).unwrap()
        })
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        Python::with_gil(|py| {
            let result = self
                .inner
                .as_ref()
                .call_method1(py, "_receive_game_update", (update,))
                .unwrap();
            result.extract::<NoAction>(py).unwrap()
        })
    }
}
