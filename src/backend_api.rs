use crate::model::{
    game_errors::GameError,
    game_logic::Game,
    player::{
        base_player::Player,
        player_actions::{
            random_actions::RandomPlayerActions, websocket_actions::WebsocketActions,
        },
    },
};

use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
pub use axum_macros::debug_handler;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{Mutex, mpsc};
use tracing::{Level, error, info};

// Define the game state. It now tracks players and the current turn.
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthParams {
    player_id: String,
    password: String,
}

pub struct WebsocketGame {
    connected_players: Arc<Mutex<BTreeMap<String, bool>>>,
    channel_for_ws_actions:
        Arc<Mutex<BTreeMap<String, (Sender<Message>, Arc<Mutex<Receiver<Message>>>)>>>,
}

impl WebsocketGame {
    pub fn new() -> WebsocketGame {
        let connected_players: Arc<Mutex<BTreeMap<String, bool>>> =
            Arc::new(Mutex::new(BTreeMap::new()));
        let channel_for_ws_actions: Arc<
            Mutex<BTreeMap<String, (Sender<Message>, Arc<Mutex<Receiver<Message>>>)>>,
        > = Arc::new(Mutex::new(BTreeMap::new()));

        WebsocketGame {
            connected_players: connected_players,
            channel_for_ws_actions: channel_for_ws_actions,
        }
    }

    pub async fn get_missing_players(&self) -> Vec<String> {
        let locked_connected_player = self.connected_players.lock().await;
        let player_ids: Vec<String> = locked_connected_player.keys().cloned().collect();

        let mut missing_players = Vec::new();
        for current_id in player_ids {
            let is_connected = locked_connected_player.get(&current_id).unwrap();
            if !is_connected {
                missing_players.push(current_id.clone());
            }
        }
        missing_players
    }
}

#[debug_handler]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<AuthParams>,
    State(state): State<Arc<Mutex<WebsocketGame>>>,
) -> impl IntoResponse {
    let player_id = params.player_id.clone();
    let password = params.password.clone();
    ws.on_upgrade(|socket| handle_socket(socket, state, player_id, password))
}

async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<Mutex<WebsocketGame>>,
    player_id: String,
    password: String,
) {
    info!("bck | New bot connecting...");
    if password != player_id {
        info!("bck | password for bot {}, was wrong", player_id);
        return;
    }

    let (state_sender, mut state_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel(1);
    let (action_sender, action_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel(1);

    let channels_for_ws_action = (state_sender, Arc::new(Mutex::new(action_receiver)));

    {
        let state_lock = state.lock().await;
        let mut map_lock = state_lock.channel_for_ws_actions.lock().await;
        map_lock.insert(player_id.clone(), channels_for_ws_action);
    }

    // for each bot, create two channels
    // 1. to send messages containing the actions received from the client-bot over the websocket to the server-bot from our logic (from action_sender to action_receiver)
    // 2. to send the game state and possible actions that that are called from the server on the bot (from state_sender to state_receiver)
    // to the websocket so it can send it to the client-bot

    info!("bck | Bot connected with ID: {}", player_id.clone());
    // todo use a tokio notify or watch here to check if the channels have been created
    // or should i just create the channels here, read them in the lobby, where i create the game and loop here forever until the bot disconnects,
    // so i could just reuse the channel over multiple games

    let connected_players = {
        let state_lock = state.lock().await;
        Arc::clone(&state_lock.connected_players)
    };

    connected_players
        .lock()
        .await
        .insert(player_id.clone(), true);

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
                info!("bck | Closing WebSocket connection of bot {}.", player_id);
                break;
            }
            _ => {
                error!(
                    "bck | received unknown message type from client of bot {}, closing WebSocket connection",
                    player_id
                );
                break;
            }
        }
    }

    info!("bck | Bot ID {} disconnected.", player_id);

    connected_players.lock().await.insert(player_id, false);

    state_receiver.close();
}

pub async fn organize_new_game(state: Arc<Mutex<WebsocketGame>>) {
    let mut missing_players = state.lock().await.get_missing_players().await;

    while missing_players.len() > 0 {
        info!("og | Waiting for missing players: {:?}", missing_players);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        missing_players = state.lock().await.get_missing_players().await;
    }

    while state.lock().await.connected_players.lock().await.len() < 2 {
        println!("og | waiting for more players to join");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    info!("og | all players joined");
    loop {
        missing_players = state.lock().await.get_missing_players().await;
        if missing_players.len() > 0 {
            info!(
                "og | The game should be interrupted and the following players removed: {:?}",
                missing_players
            );
        }

        let ws_lobby = Arc::clone(&state);
        tokio::spawn(async move {
            let ufuk_channel = {
                let lobby_lock = ws_lobby.lock().await;
                let channel_for_ws_actions = lobby_lock.channel_for_ws_actions.lock().await;
                let (ufuk_sender, ufuk_receiver) =
                    channel_for_ws_actions.get(&"ufuk".to_string()).unwrap();

                (ufuk_sender.clone(), Arc::clone(&ufuk_receiver))
            };

            let leon_channel = {
                let lobby_lock = ws_lobby.lock().await;
                let channel_for_ws_actions = lobby_lock.channel_for_ws_actions.lock().await;
                let (leon_sender, leon_receiver) =
                    channel_for_ws_actions.get(&"leon".to_string()).unwrap();

                (leon_sender.clone(), Arc::clone(&leon_receiver))
            };

            let ufuk_ws_action = WebsocketActions::new("ufuk".to_string(), ufuk_channel);
            let leon_ws_action = WebsocketActions::new("leon".to_string(), leon_channel);
            let gregor_random_action = RandomPlayerActions::new("gregor".to_string(), 25);

            let seed: u64 = 0;
            let game_handle = tokio::task::spawn_blocking(move || {
                println!("-------Default game--------\n");
                let mut game = Game::new_default_game(
                    vec![
                        String::from("ufuk"),
                        String::from("leon"),
                        String::from("gregor"),
                    ],
                    vec![
                        Box::new(ufuk_ws_action),
                        Box::new(leon_ws_action),
                        Box::new(gregor_random_action),
                    ],
                    seed,
                );

                game.num_players();
                println!("{}", game);

                game.num_players();

                let results = game.play().unwrap();

                println!("ranking: {:?}", results);
                tracing::event!(target: "results", Level::INFO, "{:?}", results);

                print!("game is done");
            });

            game_handle.await;
        });

        info!("og | It's bot ID ___'s turn.",);
        // todo, should i use this: state.game.play_one_round();

        // todo: do i want to keep this (either the sleep or in general the organize game)
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
