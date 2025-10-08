use crate::messages::actions::{
    AuctionDecision, Bidding, FromActionMessage, InitialTrade, NoAction, PlayerTurnDecision,
    SendMoney, TradeOffer, TradeOpponentDecision,
};
use crate::messages::game_updates::{AuctionRound, GameUpdate};
use crate::messages::message_protocol::{ActionMessage, StateMessage};
use crate::model::money::money::Money;
use crate::model::money::value::Value;
use crate::model::player::base_player::PlayerId;

use crate::model::player::player_actions::base_player_actions::PlayerActions;

use axum::extract::ws::{Message, Utf8Bytes};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct WebsocketActions {
    // for each bot, create two channels
    // 1. to send messages containing the actions received from the client-bot over the websocket to the server-bot from our logic (from action_sender to action_receiver)
    // 2. to send the game state and possible actions that that are called from the server on the bot (from state_sender to state_receiver)
    // to the websocket so it can send it to the client-bot

    // used by the player
    state_sender: Sender<Message>,
    action_receiver: Receiver<Message>,
    id: String,
}

impl WebsocketActions {
    pub fn new(id: String) -> (WebsocketActions, (Receiver<Message>, Sender<Message>)) {
        let (state_sender, state_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel(1);
        let (action_sender, action_receiver): (Sender<Message>, Receiver<Message>) =
            mpsc::channel(1);

        (
            WebsocketActions {
                state_sender: state_sender,
                action_receiver: action_receiver,
                id: id,
            },
            (state_receiver, action_sender),
        )
    }

    pub async fn close_connections(&mut self) {
        self.action_receiver.close();
    }

    pub fn send_and_recv<T: FromActionMessage>(&mut self, msg: StateMessage) -> T {
        println!(
            "wsp | going to send state from game to backend {}, {}",
            self.id, msg
        );
        self.state_sender
            .blocking_send(Message::Text(Utf8Bytes::from(
                serde_json::to_string(&msg).unwrap().as_str(),
            )))
            .unwrap();
        println!(
            "wsp | finished, sending state to backend {}, {}",
            self.id, msg
        );

        println!(
            "wsp | waiting for action from backend for game {}, {}",
            self.id, msg
        );
        let msg: Option<Message> = self.action_receiver.blocking_recv();
        println!("wsp | finished, receiving action from backend {}", self.id,);

        let action_msg: ActionMessage = match msg {
            Some(text) => serde_json::from_str(text.to_text().unwrap()).unwrap(),
            None => todo!("channel closed {}", self.id),
        };
        T::extract(action_msg)
    }
}

impl PlayerActions for WebsocketActions {
    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        let msg: StateMessage = StateMessage::ProvideBidding { state: state };
        self.send_and_recv(msg)
    }

    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        let msg: StateMessage = StateMessage::DrawOrTrade;
        self.send_and_recv(msg)
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        let msg: StateMessage = StateMessage::BuyOrSell { state: state };
        self.send_and_recv(msg)
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        let msg: StateMessage = StateMessage::GameUpdate { update: update };
        self.send_and_recv(msg)
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        let msg: StateMessage = StateMessage::SendMoney {
            player_id: player.clone(),
            amount: amount,
        };
        self.send_and_recv(msg)
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        let msg: StateMessage = StateMessage::RespondToTrade { offer: offer };
        self.send_and_recv(msg)
    }

    fn _trade(&mut self) -> InitialTrade {
        let msg: StateMessage = StateMessage::Trade;
        self.send_and_recv(msg)
    }
}
