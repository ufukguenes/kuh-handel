use kuh_handel_lib::messages::actions::{
    AuctionDecision, Bidding, FromActionMessage, InitialTrade, NoAction, PlayerTurnDecision,
    SendMoney, TradeOpponentDecision,
};
use kuh_handel_lib::messages::game_updates::{AuctionRound, GameUpdate, TradeOffer};
use kuh_handel_lib::messages::message_protocol::{ActionMessage, StateMessage};

use kuh_handel_lib::Value;
use kuh_handel_lib::player::base_player::PlayerId;

use kuh_handel_lib::player::player_actions::PlayerActions;
use kuh_handel_lib::player::simple_player::SimplePlayer;

use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{error, info};

pub struct WebsocketActions {
    // for each bot, create two channels
    // 1. to send messages containing the actions received from the client-bot over the websocket to the server-bot from our logic (from action_sender to action_receiver)
    // 2. to send the game state and possible actions that that are called from the server on the bot (from state_sender to state_receiver)
    // to the websocket so it can send it to the client-bot

    // used by the player
    state_sender: Sender<serde_json::Value>,
    action_receiver: Receiver<serde_json::Value>,
    id: String,
    backup_actions: SimplePlayer,
}

impl WebsocketActions {
    pub fn new(
        id: String,
        (state_sender, action_receiver): (Sender<serde_json::Value>, Receiver<serde_json::Value>),
        seed: u64,
    ) -> WebsocketActions {
        WebsocketActions {
            state_sender: state_sender,
            action_receiver: action_receiver,
            id: id.clone(),
            backup_actions: SimplePlayer::new_from_seed(id, seed),
        }
    }

    pub fn send_and_recv<T: FromActionMessage>(&mut self, msg: StateMessage) -> Option<T> {
        let mut close_channel = false;

        info!(
            "wsp | going to send state from game to backend {}, {}",
            self.id, msg
        );

        if let StateMessage::GameUpdate {
            update:
                GameUpdate::End {
                    ranking: _,
                    illegal_moves_made: _,
                },
        } = &msg
        {
            close_channel = true;
        }

        let serialized_obj = match serde_json::to_value(&msg) {
            Ok(text) => text,
            Err(_) => return None,
        };

        if self.state_sender.is_closed() {
            self.action_receiver.close();
            return None;
        }
        let try_send_to_backend = self.state_sender.blocking_send(serialized_obj);

        if let Err(e) = try_send_to_backend {
            info!("wsp | Player: {}, {}", self.id, e);
            return None;
        }

        info!(
            "wsp | finished, sending state to backend {}, {}",
            self.id, msg
        );

        if close_channel {
            self.action_receiver.close();
            let act_msg = ActionMessage::NoAction {
                decision: NoAction::Ok(),
            };
            return T::extract(act_msg);
        }

        info!(
            "wsp | waiting for action from backend for game {}, {}",
            self.id, msg
        );

        info!("wsp | waiting for lock {}", self.id,);
        let msg = self.action_receiver.blocking_recv();
        info!("wsp | finished, receiving action from backend {}", self.id,);

        let msg = match msg {
            Some(msg) => msg,
            None => return None,
        };

        let action_msg: ActionMessage = match serde_json::from_value(msg) {
            Ok(action_msg) => action_msg,
            Err(e) => {
                info!("wsp | Player: {}, {}", self.id, e);
                return None;
            }
        };

        T::extract(action_msg)
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
            None => {
                error!(
                    "wsp | {} provide_bidding switched to backup action",
                    self.id
                );
                self.backup_actions._provide_bidding(state)
            }
        }
    }

    fn _draw_or_trade(&mut self) -> PlayerTurnDecision {
        let msg: StateMessage = StateMessage::DrawOrTrade();
        let decision: Option<PlayerTurnDecision> = self.send_and_recv(msg);
        match decision {
            Some(decision) => decision,
            None => {
                error!("wsp | {} draw_or_trade switched to backup action", self.id);
                self.backup_actions._draw_or_trade()
            }
        }
    }

    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        let msg: StateMessage = StateMessage::BuyOrSell {
            state: state.clone(),
        };
        let decision: Option<AuctionDecision> = self.send_and_recv(msg);
        match decision {
            Some(decision) => decision,
            None => {
                error!("wsp | {} buy_or_sell switched to backup action", self.id);
                self.backup_actions._buy_or_sell(state)
            }
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
            None => {
                error!("wsp | {} game_update switched to backup action", self.id);
                backup_decision
            }
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
            None => {
                error!("wsp | {} send_money switched to backup action", self.id);
                self.backup_actions._send_money_to_player(player, amount)
            }
        }
    }

    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        let msg: StateMessage = StateMessage::RespondToTrade {
            offer: offer.clone(),
        };
        let decision: Option<TradeOpponentDecision> = self.send_and_recv(msg);
        match decision {
            Some(decision) => decision,
            None => {
                error!(
                    "wsp | {} respond_to_trade switched to backup action",
                    self.id
                );
                self.backup_actions._respond_to_trade(offer)
            }
        }
    }

    fn _trade(&mut self) -> InitialTrade {
        let msg: StateMessage = StateMessage::Trade();
        let decision: Option<InitialTrade> = self.send_and_recv(msg);
        match decision {
            Some(decision) => decision,
            None => {
                error!("wsp | {} trade switched to backup action", self.id);
                self.backup_actions._trade()
            }
        }
    }
}
