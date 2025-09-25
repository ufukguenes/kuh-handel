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
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{Mutex, mpsc};

pub struct WebsocketActions {
    // for each bot, create two channels
    // 1. to send messages containing the actions received from the client-bot over the websocket to the server-bot from our logic (from action_sender to action_receiver)
    // 2. to send the game state and possible actions that that are called from the server on the bot (from state_sender to state_receiver)
    // to the websocket so it can send it to the client-bot

    // used by the player
    state_sender: Sender<Message>,
    action_receiver: Receiver<Message>,

    // used by the WebSocket connection
    state_receiver: Arc<Mutex<Receiver<Message>>>,
    action_sender: Arc<Mutex<Sender<Message>>>,
}

pub type AsyncChannel = (Arc<Mutex<Receiver<Message>>>, Arc<Mutex<Sender<Message>>>);

impl WebsocketActions {
    pub fn new() -> WebsocketActions {
        let (state_sender, state_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel(1);
        let (action_sender, action_receiver): (Sender<Message>, Receiver<Message>) =
            mpsc::channel(1);

        let async_state_receiver = Arc::new(Mutex::new(state_receiver));
        let async_action_sender = Arc::new(Mutex::new(action_sender));
        WebsocketActions {
            state_sender: state_sender,
            action_receiver: action_receiver,
            state_receiver: async_state_receiver,
            action_sender: async_action_sender,
        }
    }

    pub fn get_channels(&self) -> AsyncChannel {
        (
            Arc::clone(&self.state_receiver),
            Arc::clone(&self.action_sender),
        )
    }

    pub fn close_connections(&mut self) {
        self.state_receiver.blocking_lock().close();
        self.action_receiver.close();
    }
}

impl PlayerActions for WebsocketActions {
    fn provide_bidding(&mut self, state: AuctionState) -> AuctionValue {
        self.state_sender
            .blocking_send(Message::Text(Utf8Bytes::from("ws bidding state message")));
        let msg = self.action_receiver.blocking_recv();
        match msg {
            Some(msg) => println!("ws: provide bidding {}", msg.to_text().unwrap()),
            None => println!("ws: provide bidding None"),
        }

        AuctionValue::Pass
    }

    fn draw_or_trade(&mut self) -> FirstPhaseAction {
        self.state_sender
            .blocking_send(Message::Text(Utf8Bytes::from("ws draw_or_trade state")));
        let msg = self.action_receiver.blocking_recv();
        match msg {
            Some(msg) => println!("ws: draw_or_trade {}", msg.to_text().unwrap()),
            None => println!("ws: draw_or_trade None"),
        }
        FirstPhaseAction::Draw
    }

    fn buy_or_sell(&mut self, state: AuctionState) -> AuctionAction {
        self.state_sender
            .blocking_send(Message::Text(Utf8Bytes::from("ws buy_or_sell state")));
        let msg = self.action_receiver.blocking_recv();
        match msg {
            Some(msg) => println!("ws: buy_or_sell {}", msg.to_text().unwrap()),
            None => println!("ws: buy_or_sell None"),
        }
        AuctionAction::Buy
    }
}
