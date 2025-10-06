use crate::messages::actions::{
    self, AuctionDecision, Bidding, InitialTrade, PlayerTurnDecision, TradeOffer,
    TradeOpponentDecision,
};
use crate::messages::game_updates::{AuctionRound, GameUpdate};
use crate::messages::message_protocol::{ActionMessage, StateMessage};
use crate::model::money::money::Money;
use crate::model::money::value::Value;
use crate::model::player::base_player::PlayerId;

use crate::model::player::player_actions::base_player_actions::PlayerActions;

use axum::extract::ws::{Message, Utf8Bytes};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct WebsocketActions {
    // for each bot, create two channels
    // 1. to send messages containing the actions received from the client-bot over the websocket to the server-bot from our logic (from action_sender to action_receiver)
    // 2. to send the game state and possible actions that that are called from the server on the bot (from state_sender to state_receiver)
    // to the websocket so it can send it to the client-bot

    // used by the player
    state_sender: Sender<Message>,
    action_receiver: Receiver<Message>,
}

impl WebsocketActions {
    pub fn new() -> (WebsocketActions, (Receiver<Message>, Sender<Message>)) {
        let (state_sender, state_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel(1);
        let (action_sender, action_receiver): (Sender<Message>, Receiver<Message>) =
            mpsc::channel(1);

        (
            WebsocketActions {
                state_sender: state_sender,
                action_receiver: action_receiver,
            },
            (state_receiver, action_sender),
        )
    }

    pub async fn close_connections(&mut self) {
        self.action_receiver.close();
    }

    pub fn send_and_recv(&mut self, msg: StateMessage) -> ActionMessage {
        self.state_sender
            .blocking_send(Message::Text(Utf8Bytes::from(
                serde_json::to_string(&msg).unwrap().as_str(),
            )))
            .unwrap();

        let msg: Option<Message> = self.action_receiver.blocking_recv();

        let action_msg: ActionMessage = match msg {
            Some(text) => serde_json::from_str(text.to_text().unwrap()).unwrap(),
            None => todo!("channel closed"),
        };
        action_msg
    }
}

impl PlayerActions for WebsocketActions {
    fn provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        let msg: StateMessage = StateMessage::ProvideBidding { state: state };
        let action_msg = self.send_and_recv(msg);

        match action_msg {
            ActionMessage::Bidding { decision } => {
                return decision;
            }
            _ => todo!("wrong action type"),
        }
    }

    fn draw_or_trade(&mut self) -> PlayerTurnDecision {
        let msg: StateMessage = StateMessage::DrawOrTrade;
        let action_msg = self.send_and_recv(msg);

        match action_msg {
            ActionMessage::PlayerTurnDecision { decision } => {
                return decision;
            }
            _ => todo!("wrong action type"),
        }
    }

    fn buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        let msg: StateMessage = StateMessage::BuyOrSell { state: state };
        let action_msg = self.send_and_recv(msg);

        match action_msg {
            ActionMessage::AuctionDecision { decision } => {
                return decision;
            }
            _ => todo!("wrong action type"),
        }
    }

    fn receive_game_update(&mut self, update: GameUpdate) {
        let msg: StateMessage = StateMessage::GameUpdate { update: update };
        let action_msg = self.send_and_recv(msg);

        match action_msg {
            ActionMessage::NoAction => {}
            _ => todo!("wrong action type"),
        }
    }

    fn send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> Vec<Money> {
        let msg: StateMessage = StateMessage::SendMoney {
            player_id: player.clone(),
            amount: amount,
        };
        let action_msg = self.send_and_recv(msg);

        match action_msg {
            ActionMessage::SendMoney { decision } => {
                return decision;
            }
            _ => todo!("wrong action type"),
        }
    }

    fn receive_from_player(&mut self, player: &PlayerId, money: Vec<Money>) {
        let msg: StateMessage = StateMessage::ReceiveFromPlayer {
            player_id: player.clone(),
            money: money,
        };
        let action_msg = self.send_and_recv(msg);

        match action_msg {
            ActionMessage::NoAction => {}
            _ => todo!("wrong action type"),
        }
    }

    fn respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        let msg: StateMessage = StateMessage::RespondToTrade { offer: offer };
        let action_msg = self.send_and_recv(msg);

        match action_msg {
            ActionMessage::TradeOpponentDecision { decision } => {
                return decision;
            }
            _ => todo!("wrong action type"),
        }
    }

    fn trade(&mut self) -> InitialTrade {
        let msg: StateMessage = StateMessage::Trade;
        let action_msg = self.send_and_recv(msg);

        match action_msg {
            ActionMessage::InitialTrade { decision } => {
                return decision;
            }
            _ => todo!("wrong action type"),
        }
    }
}
