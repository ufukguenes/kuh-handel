use crate::model::player::base_player::{
    AuctionAction, AuctionState, AuctionValue, FirstPhaseAction,
};
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use axum::Json;
use axum::{
    extract::{
        State,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use tokio::sync::mpsc;

pub struct WebsocketActions {
    // for each bot, create two channels
    // 1. to send messages containing the actions received from the client-bot over the websocket to the server-bot from our logic (from action_sender to action_receiver)
    // 2. to send the game state and possible actions that that are called from the server on the bot (from state_sender to state_receiver)
    // to the websocket so it can send it to the client-bot

    // used by the player
    state_sender: mpsc::Sender<Message>,
    action_receiver: mpsc::Receiver<Message>,

    // used by the WebSocket connection
    state_receiver: mpsc::Receiver<Message>,
    action_sender: mpsc::Sender<Message>,
}

impl WebsocketActions {
    pub fn get_channels(&self) -> (&mpsc::Receiver<Message>, &mpsc::Sender<Message>) {
        (&self.state_receiver, &self.action_sender)
    }

    pub fn close_connections(&mut self) {
        self.state_receiver.close();
        self.action_receiver.close();
    }
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
