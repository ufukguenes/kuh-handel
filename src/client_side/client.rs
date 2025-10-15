use std::cmp::min;
use std::panic;
use std::str::FromStr;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::messages::message_protocol::{ActionMessage, StateMessage};
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use crate::model::player::player_actions::random_actions::RandomPlayerActions;

pub struct Client {
    pub name: String,
    pub bot: RandomPlayerActions,
    pub print_indent_size: usize,
}

// todo adjust client connection like in backend
impl Client {
    const INDENT_MULTIPLIER: usize = 1000;
    const COLUMN_BUFFER: usize = 5;

    fn print_in_columns(&self, text: String) {
        let print_str: String = format!("{}{}!", self.indent_space(), text);
        println!(
            "{}",
            &print_str[..min(
                print_str.len(),
                self.print_indent_size * Self::INDENT_MULTIPLIER + Self::INDENT_MULTIPLIER
                    - Self::COLUMN_BUFFER
            )]
        );
    }

    pub async fn start(mut self) {
        let (ws_stream, _) = connect_async(format!(
            "ws://127.0.0.1:3000/game?player_id={}&password={}",
            self.name, self.name
        ))
        .await
        .expect("Failed to connect");

        self.print_in_columns(format!("Connected to server!"));

        let (mut send, mut recv) = ws_stream.split();

        // Spawn a task to listen for incoming messages
        while let Some(msg) = recv.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let state_msg: StateMessage = serde_json::from_str(&text).unwrap();

                    self.print_in_columns(format!("bot {} received message: {}", self.name, text));

                    let action_msg: ActionMessage = self.bot.map_to_action(state_msg);

                    self.print_in_columns(format!(
                        "bot {} picked action: {}",
                        self.name,
                        serde_json::to_string(&action_msg).unwrap()
                    ));

                    let _ = send
                        .send(Message::Text(serde_json::to_string(&action_msg).unwrap()))
                        .await;

                    self.print_in_columns(format!("bot {}, finished sending action", self.name));
                }

                Ok(Message::Close(_)) => {
                    self.print_in_columns("Connection closed by server".to_string());
                    break;
                }
                Ok(other) => {
                    self.print_in_columns(format!("Received other message: {:?}", other));
                }
                Err(e) => {
                    self.print_in_columns(format!("Error: {}", e));
                    break;
                }
            }
            self.print_in_columns(format!("waiting for next action request"));
        }
        self.print_in_columns(format!(
            "ranking: {:?}",
            self.bot
                .final_ranking()
                .iter()
                .map(|ranking| (ranking.0.name.clone(), ranking.1.to_string()))
                .collect::<Vec<_>>(),
        ));
    }

    pub fn indent_space(&self) -> String {
        String::from_str(" ")
            .unwrap()
            .repeat(Self::INDENT_MULTIPLIER * self.print_indent_size)
    }
}
