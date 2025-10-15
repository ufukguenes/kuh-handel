use std::sync::Arc;

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
use crate::model::player::player_actions::random_actions::RandomPlayerActions;

use axum::extract::ws::{Message, Utf8Bytes};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{Mutex, mpsc};
use tracing::{error, info};

pub struct WebsocketActions {
    // for each bot, create two channels
    // 1. to send messages containing the actions received from the client-bot over the websocket to the server-bot from our logic (from action_sender to action_receiver)
    // 2. to send the game state and possible actions that that are called from the server on the bot (from state_sender to state_receiver)
    // to the websocket so it can send it to the client-bot

    // used by the player
    state_sender: Sender<Message>,
    action_receiver: Arc<Mutex<Receiver<Message>>>,
    id: String,
    backup_actions: RandomPlayerActions,
}

impl WebsocketActions {
    pub fn new(
        id: String,
        (state_sender, action_receiver): (Sender<Message>, Arc<Mutex<Receiver<Message>>>),
    ) -> WebsocketActions {
        WebsocketActions {
            state_sender: state_sender,
            action_receiver: action_receiver,
            id: id.clone(),
            backup_actions: RandomPlayerActions::new(id, 42),
        }
    }

    pub async fn close_connections(&mut self) {
        self.action_receiver.lock().await.close();
    }

    pub fn send_and_recv<T: FromActionMessage>(&mut self, msg: StateMessage) -> Option<T> {
        info!(
            "wsp | going to send state from game to backend {}, {}",
            self.id, msg
        );
        self.state_sender
            .blocking_send(Message::Text(Utf8Bytes::from(
                serde_json::to_string(&msg).unwrap().as_str(),
            )))
            .unwrap();
        info!(
            "wsp | finished, sending state to backend {}, {}",
            self.id, msg
        );

        info!(
            "wsp | waiting for action from backend for game {}, {}",
            self.id, msg
        );
        let msg: Option<Message> = self.action_receiver.blocking_lock().blocking_recv();
        info!("wsp | finished, receiving action from backend {}", self.id,);

        let action_msg: ActionMessage = match msg {
            Some(text) => serde_json::from_str(text.to_text().unwrap()).unwrap(),
            None => return None,
        };
        Some(T::extract(action_msg))
    }
}

impl PlayerActions for WebsocketActions {
    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        let msg: StateMessage = StateMessage::ProvideBidding {
            state: state.clone(),
        };

        let decision: Option<Bidding> = self.send_and_recv(msg);
        match decision {
            Some(decision) => decision,
            None => self.backup_actions._provide_bidding(state),
        }
    }

    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        let msg: StateMessage = StateMessage::DrawOrTrade;
        let decision: Option<PlayerTurnDecision> = self.send_and_recv(msg);
        match decision {
            Some(decision) => decision,
            None => self.backup_actions._draw_or_trade(),
        }
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        let msg: StateMessage = StateMessage::BuyOrSell {
            state: state.clone(),
        };
        let decision: Option<AuctionDecision> = self.send_and_recv(msg);
        match decision {
            Some(decision) => decision,
            None => self.backup_actions._buy_or_sell(state),
        }
    }

    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction {
        let msg: StateMessage = StateMessage::GameUpdate {
            update: update.clone(),
        };

        // keep the backup, up to date so that it can jump in at any time
        let backup_decision = self.backup_actions._receive_game_update(update.clone());

        let decision: Option<NoAction> = self.send_and_recv(msg);
        match decision {
            Some(decision) => decision,
            None => backup_decision,
        }
    }

    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney {
        let msg: StateMessage = StateMessage::SendMoney {
            player_id: player.clone(),
            amount: amount,
        };
        let decision: Option<SendMoney> = self.send_and_recv(msg);
        match decision {
            Some(decision) => decision,
            None => self.backup_actions._send_money_to_player(player, amount),
        }
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        let msg: StateMessage = StateMessage::RespondToTrade {
            offer: offer.clone(),
        };
        let decision: Option<TradeOpponentDecision> = self.send_and_recv(msg);
        match decision {
            Some(decision) => decision,
            None => self.backup_actions._respond_to_trade(offer),
        }
    }

    fn _trade(&mut self) -> InitialTrade {
        let msg: StateMessage = StateMessage::Trade;
        let decision: Option<InitialTrade> = self.send_and_recv(msg);
        match decision {
            Some(decision) => decision,
            None => self.backup_actions._trade(),
        }
    }
}
