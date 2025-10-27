use crate::PlayerId;
use crate::Value;
use kuh_handel_lib::messages::{actions::*, game_updates::*, message_protocol::*};
use kuh_handel_lib::player::player_actions::PlayerActions as CorePlayerActions;
use kuh_handel_lib::player::random_player::RandomPlayerActions as CoreRandomPlayerActions;
use pyo3::prelude::*;

#[pymodule]
pub fn player_actions_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<PlayerActions>();

    Ok(())
}
#[pyclass(unsendable)]
#[derive(Clone)]
pub struct PlayerActions {
    inner: CoreRandomPlayerActions,
}

#[pymethods]
impl PlayerActions {
    #[new]
    pub fn new(id: String) -> Self {
        PlayerActions {
            inner: CoreRandomPlayerActions::new(id, 0),
        }
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

    //todo remove randomplayer, the clone from randomplayer
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        self.inner._draw_or_trade()
    }

    fn _trade(&mut self) -> InitialTrade {
        self.inner._trade()
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        self.inner._provide_bidding(state)
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        self.inner._buy_or_sell(state)
    }

    fn _send_money_to_player(&mut self, player: PlayerId, amount: Value) -> SendMoney {
        self.inner._send_money_to_player(&player.clone(), amount)
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        self.inner._respond_to_trade(offer)
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        self.inner._receive_game_update(update)
    }
}

pub struct RustPlayer {
    pub inner: PlayerActions,
}

impl CorePlayerActions for RustPlayer {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        self.inner._draw_or_trade()
    }

    fn _trade(&mut self) -> InitialTrade {
        self.inner._trade()
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        self.inner._provide_bidding(state)
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        self.inner._buy_or_sell(state)
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        self.inner._send_money_to_player(player.clone(), amount)
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        self.inner._respond_to_trade(offer)
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        self.inner._receive_game_update(update)
    }
}
