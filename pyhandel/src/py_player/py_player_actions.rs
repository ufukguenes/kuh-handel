use crate::py_messages::py_actions::*;
use crate::py_messages::py_game_updates::*;
use crate::py_messages::py_message_protocol::*;
use crate::PlayerId;
use crate::Value;
use kuh_handel_lib::player::player_actions::PlayerActions as CorePlayerActions;
use pyo3::prelude::*;

#[pymodule]
pub fn player_actions_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<PlayerActions>();

    Ok(())
}
#[pyclass(unsendable)]
#[derive(Clone)]
pub struct PlayerActions {}

#[pymethods]
impl PlayerActions {
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

    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        todo!()
    }

    fn _trade(&mut self) -> InitialTrade {
        todo!()
    }

    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        todo!()
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        todo!()
    }

    fn _send_money_to_player(&mut self, player: PlayerId, amount: Value) -> SendMoney {
        todo!()
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        todo!()
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        todo!()
    }
}

struct CorePlayer {
    inner: PlayerActions,
}

impl CorePlayerActions for CorePlayer {
    fn _draw_or_trade(&mut self) -> kuh_handel_lib::messages::actions::PlayerTurnDecision {
        self.inner._draw_or_trade()
    }

    fn _trade(&mut self) -> kuh_handel_lib::messages::actions::InitialTrade {
        self.inner._trade()
    }

    fn _provide_bidding(
        &mut self,
        state: kuh_handel_lib::messages::game_updates::AuctionRound,
    ) -> kuh_handel_lib::messages::actions::Bidding {
        self.inner._provide_bidding()
    }

    fn _buy_or_sell(
        &mut self,
        state: kuh_handel_lib::messages::game_updates::AuctionRound,
    ) -> kuh_handel_lib::messages::actions::AuctionDecision {
        self.inner._buy_or_sell()
    }

    fn _send_money_to_player(
        &mut self,
        player: &kuh_handel_lib::player::base_player::PlayerId,
        amount: kuh_handel_lib::Value,
    ) -> kuh_handel_lib::messages::actions::SendMoney {
        self.inner._send_money_to_player()
    }

    fn _respond_to_trade(
        &mut self,
        offer: kuh_handel_lib::messages::actions::TradeOffer,
    ) -> kuh_handel_lib::messages::actions::TradeOpponentDecision {
        self.inner._respond_to_trade()
    }

    fn _receive_game_update(
        &mut self,
        update: kuh_handel_lib::messages::game_updates::GameUpdate,
    ) -> kuh_handel_lib::messages::actions::NoAction {
        self.inner._receive_game_update()
    }
}
