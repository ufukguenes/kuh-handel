use futures_util::{SinkExt, StreamExt};
use reqwest::Client as HttpClient;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::messages::message_protocol::{ActionMessage, StateMessage};
use crate::player::player_actions::PlayerActions;

/// This Client is used to connect a Bot too the kuh-handel server
pub struct Client {
    /// name of the client. Should be identical to the name the bot uses
    pub name: String,

    /// user defined token for authentication
    pub token: String,

    /// the actual bot that provides the actions for the game. When providing actions with its name, that name should be the same as the clients name
    pub bot: Box<dyn PlayerActions + Send + Sync>,

    /// the url to connect to where the game is hosted, e.g. s://ufuk-guenes.com or ://127.0.0.1:2000 if you want to connect to a locally hosted server
    /// the url requires "s://" for a secure connection
    pub base_url: String,
    last_ranking: Vec<(String, usize)>,
}

impl Client {
    /// Creates a new Client
    pub fn new(
        name: String,
        token: String,
        bot: Box<dyn PlayerActions + Send + Sync>,
        base_url: String,
    ) -> Self {
        Client {
            name,
            token,
            bot,
            base_url,
            last_ranking: Vec::new(),
        }
    }

    /// Registers a new player with the provided name and token. This only needs to be called once
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

    /// Connects to the server and tries to play one round.
    /// `game_type_url` - the type of game to connect to. Possible choices:
    ///                   game: waits for other players to join and then plays against those. These games are counted in the results
    ///                   random_game: only play against random bots provided by the server. Use this for testing if your bot can play valid games. NOT counted in the results
    pub async fn play_one_round(&mut self, game_type_url: String) {
        let (ws_stream, _) = connect_async(format!(
            "ws{}/kuh-handel/{}?player_id={}&token={}",
            self.base_url, game_type_url, self.name, self.token
        ))
        .await
        .expect("Failed to connect");

        println!("Connected to server!");

        let (mut send, mut recv) = ws_stream.split();

        let ctrl_c_signal = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c_signal);

        // Spawn a task to listen for incoming messages
        loop {
            // println!("waiting for next action request");
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
                // println!("bot {} received message: {}", self.name, state_message);
                action_msg = self.bot.map_to_action(state_message);

                let state_message: StateMessage = serde_json::from_str(&text).unwrap();
                if let StateMessage::GameUpdate {
                    update: crate::messages::game_updates::GameUpdate::End { ranking },
                } = &state_message
                {
                    self.last_ranking = ranking.clone();
                    break;
                };
            }

            let action_str = serde_json::to_string(&action_msg).unwrap();
            // println!("bot {} picked action: {}", self.name, action_str);
            let message: Message = Message::Text(action_str);

            let send_status = send.send(message).await;

            match send_status {
                Ok(_) => (), //println!("action of bot {} has been send to game", self.name,),
                Err(_) => {
                    println!(
                        "failure sending action of bot {} to game, closing connection",
                        self.name,
                    );
                    break;
                }
            }

            // println!("bot {}, finished sending action", self.name);
        }

        let res = send.close().await;
        println!("res: {}, {:?}", self.name, self.last_ranking);
    }
}
