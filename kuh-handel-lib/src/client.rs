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

    pub async fn start(mut self) {
        let (ws_stream, _) = connect_async(format!(
            "ws{}/kuh-handel/game?player_id={}&token={}",
            self.base_url, self.name, self.token
        ))
        .await
        .expect("Failed to connect");

        println!("Connected to server!");

        let (mut send, mut recv) = ws_stream.split();

        // Spawn a task to listen for incoming messages
        loop {
            println!("waiting for next action request");
            let msg = match recv.next().await {
                Some(msg) => msg,
                None => {
                    println!("game closed connection to game, ending loop");
                    break;
                }
            };

            let msg_type = match msg {
                Ok(msg_type) => msg_type,
                Err(e) => {
                    println!("error receiving from game: {}", e);
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
                    println!("Received other message: {:?}", other);
                    break;
                }
            };

            let action_msg: ActionMessage;
            {
                let state_message: StateMessage = serde_json::from_str(&text).unwrap();
                println!("bot {} received message: {}", self.name, state_message);

                action_msg = self.bot.map_to_action(state_message);
            }

            println!(
                "bot {} picked action: {}",
                self.name,
                serde_json::to_string(&action_msg).unwrap()
            );
            let send_status = send
                .send(Message::Text(serde_json::to_string(&action_msg).unwrap()))
                .await;

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
            if self.bot.final_ranking().len() > 0 {
                let _ = send.close().await;
                break;
            }
        }

        println!(
            "ranking: {:?}",
            self.bot
                .final_ranking()
                .iter()
                .map(|ranking| (ranking.0.name.clone(), ranking.1))
                .collect::<Vec<_>>(),
        );
    }
}
