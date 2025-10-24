use crate::model::match_making::WebsocketLobby;
use axum::{
    extract::{
        Query, State,
        ws::{CloseCode, CloseFrame, Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{Html, IntoResponse},
};
pub use axum_macros::debug_handler;

use std::{collections::BTreeMap, sync::Arc};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{Mutex, mpsc};

use tracing::{error, info};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AuthParams {
    player_id: String,
    token: String,
}

#[derive(Clone)]
pub struct JsonLog<T> {
    pub data: Arc<Mutex<BTreeMap<String, T>>>,
    path: String,
}

impl<T> JsonLog<T>
where
    T: Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static,
{
    pub async fn new(path: String) -> Result<Self, Box<dyn std::error::Error>> {
        let new_auth = JsonLog {
            data: Arc::new(Mutex::new(BTreeMap::new())),
            path: path,
        };

        let _ = new_auth.to_file().await?;

        Ok(new_auth)
    }

    pub async fn from_file(path: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(&path).await?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let data: BTreeMap<String, T> = serde_json::from_str(&contents)?;
        Ok(JsonLog {
            data: Arc::new(Mutex::new(data)),
            path: path,
        })
    }

    pub async fn to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.data.lock().await;

        let mut json = serde_json::to_string(&*data)?;
        json = json.replace("],", "],\n");
        json = json.replace("{", "{\n");
        json = json.replace("}", "\n}");

        let mut file = File::create(&self.path).await?;
        file.write_all(json.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }

    pub async fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        let data = self.data.lock().await;

        let json = serde_json::to_string(&*data)?;
        Ok(json)
    }
}

async fn authenticate(authentication: JsonLog<String>, auth_params: &AuthParams) -> bool {
    let credentials = authentication.data.lock().await;

    match credentials.get(&auth_params.player_id) {
        Some(stored_token) => stored_token == &auth_params.token,
        None => false,
    }
}

pub async fn register_handler(
    authentication: State<JsonLog<String>>,
    Query(params): Query<AuthParams>,
) -> Result<impl IntoResponse, StatusCode> {
    {
        let mut credentials = authentication.data.lock().await;
        if credentials.contains_key(&params.player_id) {
            info!(
                "bck | Registration failed: Player ID {} already exists.",
                params.player_id
            );
            return Err(StatusCode::CONFLICT);
        }
        credentials.insert(params.player_id.clone(), params.token);
    }

    let _ = authentication.to_file().await;

    info!(
        "bck | Successfully registered new player: {}",
        params.player_id.clone()
    );
    Ok(StatusCode::CREATED)
}

pub async fn stats_handler(State(game_results): State<JsonLog<Vec<usize>>>) -> String {
    match game_results.to_json().await {
        Ok(json) => json,
        Err(e) => e.to_string(),
    }
}

#[debug_handler]
pub async fn games_per_second_handler(State(state): State<WebsocketLobby>) -> Html<String> {
    axum::response::Html(format!("{}", state.games_per_second().await))
}

#[debug_handler]
pub async fn pvp_websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<AuthParams>,
    State(state): State<(WebsocketLobby, JsonLog<String>)>,
) -> impl IntoResponse {
    let player_id = params.player_id.clone();
    let (ws_lobby, authentication) = state;

    if !authenticate(authentication, &params).await {
        info!("bck | Authentication failed for player: {}", player_id);
        return StatusCode::UNAUTHORIZED.into_response();
    }

    info!("bck | Player {} authenticated successfully.", player_id);
    ws.on_upgrade(|socket| handle_socket(socket, ws_lobby, player_id))
}

// todo this is duplicate code
#[debug_handler]
pub async fn random_websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<AuthParams>,
    State(state): State<(WebsocketLobby, JsonLog<String>)>,
) -> impl IntoResponse {
    let player_id = params.player_id.clone();
    let (ws_lobby, authentication) = state;

    if !authenticate(authentication, &params).await {
        info!("bck | Authentication failed for player: {}", player_id);
        return StatusCode::UNAUTHORIZED.into_response();
    }

    info!("bck | Player {} authenticated successfully.", player_id);
    ws.on_upgrade(|socket| handle_socket(socket, ws_lobby, player_id))
}

async fn handle_socket(mut socket: WebSocket, lobby: WebsocketLobby, player_id: String) {
    info!("bck | New bot connecting...");

    let arc_channels_for_ws_actions = Arc::clone(&lobby.channels_for_ws_actions);

    if arc_channels_for_ws_actions
        .lock()
        .await
        .get(&player_id)
        .is_some()
    {
        info!(
            "bck | Already connected bot tried to connect again {}",
            player_id
        );
        return;
    }

    let (state_sender, mut state_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel(1);
    let (action_sender, action_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel(1);

    let channels_for_this_bot = (state_sender, Arc::new(Mutex::new(action_receiver)));

    arc_channels_for_ws_actions
        .lock()
        .await
        .insert(player_id.clone(), channels_for_this_bot);

    // for each bot, create two channels
    // 1. to send messages containing the actions received from the client-bot over the websocket to the server-bot from our logic (from action_sender to action_receiver)
    // 2. to send the game state and possible actions that that are called from the server on the bot (from state_sender to state_receiver)
    // to the websocket so it can send it to the client-bot

    info!("bck | Bot connected with ID: {}", player_id.clone());

    loop {
        // send state and possible actions to client-bot

        info!(
            "bck | waiting to receive game state info for bot {}",
            player_id
        );
        let recv_state: Option<Message> = state_receiver.recv().await;

        let state_msg = match recv_state {
            Some(msg) => {
                info!(
                    "bck | received game state info for bot {}: {}",
                    player_id,
                    msg.to_text().unwrap()
                );
                msg
            }
            None => {
                info!(
                    "bck | game closed connection to bot {}, ending loop",
                    player_id
                );
                break;
            }
        };

        info!("bck | waiting to send state to client of bot {}", player_id);
        if let Err(e) = socket.send(state_msg).await {
            error!("bck | Error sending message: {}", e);
            break;
        };

        info!(
            "bck | finished sending state to client of bot {}",
            player_id
        );

        let action_msg = socket.recv().await;

        let action_msg = match action_msg {
            Some(Ok(msg)) => {
                info!(
                    "bck | bot {} action: {}",
                    player_id,
                    msg.clone().to_text().unwrap()
                );
                info!(
                    "bck | finished receiving action from client of bot {}",
                    player_id
                );
                msg
            }
            Some(Err(e)) => {
                error!("bck | error receiving from bot {}: {}", player_id, e);
                break;
            }
            None => {
                info!("bck | bot {} disconnected.", player_id);
                break;
            }
        };

        match action_msg {
            Message::Text(_) => match action_sender.send(action_msg).await {
                Ok(_) => {
                    info!("bck | action of bot {} has been send to game", player_id)
                }
                Err(_) => {
                    error!(
                        "bck | failure sending action of bot {} to game, closing connection",
                        player_id
                    );
                    break;
                }
            },

            Message::Close(_) => {
                let _ = action_sender.send(action_msg).await;
                info!("bck | Closing WebSocket connection of bot {}.", player_id);
                break;
            }
            _ => {
                info!(
                    "bck | received unknown message type from client of bot {}, closing WebSocket connection",
                    player_id
                );
                break;
            }
        }
    }

    action_sender.closed().await;
    state_receiver.close();

    arc_channels_for_ws_actions
        .lock()
        .await
        .remove(&player_id.clone());

    let close_msg = Message::Close(Some(CloseFrame {
        code: 100 as CloseCode,
        reason: "".into(),
    }));

    if let Err(e) = socket.send(close_msg).await {
        tracing::info!(
            "bck | Failed to send final close frame to bot {}: {}",
            player_id,
            e
        );
    }
    info!("bck | Bot ID {} disconnected.", player_id);
}
