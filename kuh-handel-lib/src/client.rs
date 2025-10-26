use futures_util::{SinkExt, StreamExt};
use reqwest::Client as HttpClient;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::messages::message_protocol::{ActionMessage, StateMessage};
use crate::player::player_actions::PlayerActions;
use crate::player::random_player::RandomPlayerActions;

#[derive(Debug)]
pub struct Client {
    pub name: String,
    pub token: String,
    pub bot: RandomPlayerActions,
    pub base_url: String,
}

impl Client {
    pub async fn register(&self) -> Result<(), Box<dyn std::error::Error>> {
        let http = HttpClient::new();
        println!("Registering bot {} ...", self.name);
        let response = http
            .post(format!(
                "http{}/kuh-handel/register?player_id={}&token={}",
                self.base_url, self.name, self.token
            ))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Registration failed: {:?}", response.status()).into());
        };

        println!("Successfully registered bot {}", self.name);

        Ok(())
    }

    pub async fn play_one_round(&mut self) {
        let (ws_stream, _) = connect_async(format!(
            "ws{}/kuh-handel/game?player_id={}&token={}",
            self.base_url, self.name, self.token
        ))
        .await
        .expect("Failed to connect");

        println!("Connected to server!");

        let (mut send, mut recv) = ws_stream.split();

        let ctrl_c_signal = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c_signal);

        // Spawn a task to listen for incoming messages
        loop {
            println!("waiting for next action request");
            let msg = tokio::select! {
                msg = recv.next() => {
                    match msg {
                        Some(msg) => msg,
                        None => {
                            println!("game closed connection to game, ending loop {}", self.name);
                            break;
                        }
                    }
                },
                _ = &mut ctrl_c_signal => {
                    println!("keyboard interrupt, ending loop {}", self.name);
                    break;
                }
            };

            let msg_type = match msg {
                Ok(msg_type) => msg_type,
                Err(e) => {
                    println!("error receiving from game: {}, {}", self.name, e);
                    break;
                }
            };

            let text = match msg_type {
                Message::Text(text) => text,

                Message::Close(_) => {
                    println!("Connection closed by server");
                    break;
                }
                other => {
                    println!("Received other message: {}, {:?}", self.name, other);
                    break;
                }
            };

            let action_msg: ActionMessage;
            {
                let state_message: StateMessage = serde_json::from_str(&text).unwrap();
                println!("bot {} received message: {}", self.name, state_message);

                if let StateMessage::GameUpdate {
                    update: crate::messages::game_updates::GameUpdate::End { ranking: _ },
                } = &state_message
                {
                    break;
                };

                action_msg = self.bot.map_to_action(state_message);
            }

            let action_str = serde_json::to_string(&action_msg).unwrap();
            println!("bot {} picked action: {}", self.name, action_str);
            let message: Message = Message::Text(action_str);

            let send_status = send.send(message).await;

            match send_status {
                Ok(_) => println!("action of bot {} has been send to game", self.name,),
                Err(_) => {
                    println!(
                        "failure sending action of bot {} to game, closing connection",
                        self.name,
                    );
                    break;
                }
            }

            println!("bot {}, finished sending action", self.name);
        }

        let res = send.close().await;
        println!("res: {}, {:?}", self.name, res);

        println!(
            "ranking: {:?}",
            self.bot
                .final_ranking()
                .iter()
                .map(|ranking| (ranking.0.clone(), ranking.1))
                .collect::<Vec<_>>(),
        );
    }
}
