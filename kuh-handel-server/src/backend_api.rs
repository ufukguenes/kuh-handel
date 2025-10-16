use crate::model::game_logic::Game;
use crate::server_side_player::websocket_actions::WebsocketActions;

use kuh_handel_lib::player::player_actions::PlayerActions;
use kuh_handel_lib::player::random_player::RandomPlayerActions;

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
use tokio::sync::{Mutex, mpsc};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tracing::{Level, error, info};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthParams {
    player_id: String,
    password: String,
}

pub struct WebsocketLobby {
    channels_for_ws_actions:
        Arc<Mutex<BTreeMap<String, (Sender<Message>, Arc<Mutex<Receiver<Message>>>)>>>,
}

impl WebsocketLobby {
    pub fn new() -> WebsocketLobby {
        WebsocketLobby {
            channels_for_ws_actions: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

#[debug_handler]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<AuthParams>,
    State(state): State<Arc<Mutex<WebsocketLobby>>>,
) -> impl IntoResponse {
    let player_id = params.player_id.clone();
    let password = params.password.clone();
    ws.on_upgrade(|socket| handle_socket(socket, state, player_id, password))
}

async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<Mutex<WebsocketLobby>>,
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

    let channels_for_this_bot = (state_sender, Arc::new(Mutex::new(action_receiver)));

    let arc_channels_for_ws_actions = {
        let state_lock = state.lock().await;
        Arc::clone(&state_lock.channels_for_ws_actions)
    };

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

    arc_channels_for_ws_actions
        .lock()
        .await
        .remove(&player_id.clone());

    state_receiver.close();
}

pub async fn organize_new_game(state: Arc<Mutex<WebsocketLobby>>) {
    // todo: better match making
    while state
        .lock()
        .await
        .channels_for_ws_actions
        .lock()
        .await
        .len()
        < 4
    {
        info!("og | waiting for more players to join");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    info!("og | enough players joined");
    loop {
        // todo how to handle if player drops connection? -> just use the backup action in the websocket actions?

        info!("og | creating new round of games");

        let ws_lobby = Arc::clone(&state);
        let first_game = spawn_game(
            ws_lobby,
            vec![String::from("ufuk"), String::from("leon")],
            vec![String::from("gregor")],
        );

        let ws_lobby = Arc::clone(&state);
        let second_game = spawn_game(
            ws_lobby,
            vec![String::from("johannes"), String::from("viola")],
            vec![String::from("fiete")],
        );

        let _wait = first_game.await;
        let _wait = second_game.await;

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

pub async fn spawn_game(
    state: Arc<Mutex<WebsocketLobby>>,
    ws_players: Vec<String>,
    random_players: Vec<String>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut all_ids: Vec<String> = Vec::new();
        all_ids.extend(ws_players.clone());
        all_ids.extend(random_players.clone());

        let mut ws_actions: Vec<WebsocketActions> = Vec::new();

        {
            let lobby_lock = state.lock().await;
            let channel_for_ws_actions = lobby_lock.channels_for_ws_actions.lock().await;

            for id in &ws_players {
                let (sender, receiver) = channel_for_ws_actions.get(id).unwrap();
                let channels = (sender.clone(), Arc::clone(&receiver));

                ws_actions.push(WebsocketActions::new(id.clone(), channels));
            }
        }

        let mut random_actions: Vec<RandomPlayerActions> = Vec::new();
        for id in ws_players {
            random_actions.push(RandomPlayerActions::new(id.clone(), 25)); //todo change see
        }

        let seed: u64 = 0; //todo change seed
        let game_handle = tokio::task::spawn_blocking(move || {
            println!("-------Default game--------\n");
            let mut all_actions: Vec<Box<dyn PlayerActions>> = Vec::new();
            all_actions.extend(
                ws_actions
                    .into_iter()
                    .map(|action: WebsocketActions| Box::new(action) as Box<dyn PlayerActions>),
            );
            all_actions.extend(
                random_actions
                    .into_iter()
                    .map(|action: RandomPlayerActions| Box::new(action) as Box<dyn PlayerActions>),
            );

            let mut game = Game::new_default_game(all_ids, all_actions, seed);
            game.num_players();
            println!("{}", game);

            game.num_players();

            let results = game.play().unwrap();

            println!("ranking: {:?}", results);
            tracing::event!(target: "results", Level::INFO, "{:?}", results);

            print!("game is done");
        });

        let _ = game_handle.await;
    })
}
