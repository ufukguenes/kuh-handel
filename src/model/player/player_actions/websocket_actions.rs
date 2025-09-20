use crate::model::player::base_player::{
    AuctionAction, AuctionState, AuctionValue, FirstPhaseAction,
};
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use axum::{
    extract::{
        State,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use tokio::sync::mpsc;

pub struct WebsocketActions {
    sender: mpsc::Sender<Message>,
}

impl PlayerActions for WebsocketActions {
    fn provide_bidding(&mut self, state: AuctionState) -> AuctionValue {
        todo!()
    }

    fn draw_or_trade(&mut self) -> FirstPhaseAction {
        todo!()
    }

    fn buy_or_sell(&mut self, state: AuctionState) -> AuctionAction {
        todo!()
    }
}
