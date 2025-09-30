use crate::model::money::money::Money;
use crate::model::money::value::Value;
use crate::model::player::base_player::PlayerId;
use crate::model::player::player_actions::actions::{TradeOffer, TradeOpponentDecision};
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use crate::model::player::player_actions::game_updates::Bidding;
use crate::player_actions::actions::{AuctionDecision, PlayerTurnDecision};
use crate::player_actions::game_updates::{AuctionRound, GameUpdate};
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
}

impl PlayerActions for WebsocketActions {
    fn provide_bidding(&mut self, state: AuctionRound) -> Bidding {
        self.state_sender
            .blocking_send(Message::Text(Utf8Bytes::from("ws bidding state message")));
        let msg = self.action_receiver.blocking_recv();
        match msg {
            Some(msg) => println!("ws: provide bidding {}", msg.to_text().unwrap()),
            None => println!("ws: provide bidding None"),
        }

        Bidding::Pass
    }

    fn draw_or_trade(&mut self) -> PlayerTurnDecision {
        println!("Trying to send draw or trade");
        self.state_sender
            .blocking_send(Message::Text(Utf8Bytes::from("ws draw_or_trade state")));
        println!("has send draw or trade");
        println!("trying to receive draw or trade");
        let msg = self.action_receiver.blocking_recv();
        println!("has received draw or trade");
        match msg {
            Some(msg) => println!("ws: draw_or_trade {}", msg.to_text().unwrap()),
            None => println!("ws: draw_or_trade None"),
        }
        PlayerTurnDecision::Draw
    }

    fn buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision {
        self.state_sender
            .blocking_send(Message::Text(Utf8Bytes::from("ws buy_or_sell state")));
        let msg = self.action_receiver.blocking_recv();
        match msg {
            Some(msg) => println!("ws: buy_or_sell {}", msg.to_text().unwrap()),
            None => println!("ws: buy_or_sell None"),
        }
        AuctionDecision::Buy
    }

    fn receive_game_update(&mut self, update: GameUpdate) {
        todo!()
    }

    fn send_money_to_player(&mut self, player: PlayerId, amount: Value) -> Vec<Money> {
        todo!()
    }

    fn receive_from_player(&mut self, player: PlayerId, money: Vec<Money>) {
        todo!()
    }

    fn respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision {
        todo!()
    }
}
