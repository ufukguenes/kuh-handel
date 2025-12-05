use kuh_handel_lib::messages::{actions::*, game_updates::*, message_protocol::*};
use kuh_handel_lib::player::base_player::PlayerId;
use kuh_handel_lib::player::player_actions::PlayerActions;
use kuh_handel_lib::player::simple_player::SimplePlayer as CorePlayer;
use kuh_handel_lib::Value;
use pyo3::prelude::*;

#[pymodule]
pub fn simple_player_module_entry(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use super::*;
    m.add_class::<SimplePlayer>();

    Ok(())
}

#[pyclass]
pub struct SimplePlayer {
    pub inner: CorePlayer,
}

#[pymethods]
impl SimplePlayer {
    #[new]
    pub fn new(id: String, aggressiveness: f32) -> Self {
        SimplePlayer {
            inner: CorePlayer::new(id, aggressiveness),
        }
    }

    pub fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        self.inner._draw_or_trade()
    }

    pub fn _trade(&mut self) -> InitialTrade {
        self.inner._trade()
    }

    pub fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        self.inner._provide_bidding(state)
    }

    pub fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        self.inner._buy_or_sell(state)
    }

    pub fn _send_money_to_player(&mut self, player: PlayerId, amount: Value) -> SendMoney {
        self.inner._send_money_to_player(&player, amount)
    }

    pub fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        self.inner._respond_to_trade(offer)
    }

    pub fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        self.inner._receive_game_update(update)
    }
}
