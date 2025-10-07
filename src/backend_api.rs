use crate::model::game_errors::GameError;

use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
pub use axum_macros::debug_handler;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

// Define the game state. It now tracks players and the current turn.
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthParams {
    player_id: String,
}

pub struct WebsocketGame {
    connected_players: Arc<Mutex<HashMap<String, bool>>>,
    channel_per_player: Arc<Mutex<HashMap<String, (Receiver<Message>, Sender<Message>)>>>,
}

impl WebsocketGame {
    pub async fn new(
        websocket_channels_per_player: Arc<
            Mutex<HashMap<String, (Receiver<Message>, Sender<Message>)>>,
        >,
    ) -> Result<WebsocketGame, GameError> {
        let connected_players: Arc<Mutex<HashMap<String, bool>>> =
            Arc::new(Mutex::new(HashMap::new()));

        for player_id in websocket_channels_per_player.lock().await.keys() {
            connected_players
                .lock()
                .await
                .insert(player_id.clone(), false);
        }

        Result::Ok(WebsocketGame {
            connected_players: connected_players,
            channel_per_player: websocket_channels_per_player,
        })
    }

    pub async fn get_missing_players(&self) -> Vec<String> {
        let mut missing_players = Vec::new();
        let locked_connected_player = self.connected_players.lock().await;
        for player_id in self.channel_per_player.lock().await.keys() {
            let is_connected = locked_connected_player.get(player_id).unwrap();
            if !is_connected {
                missing_players.push(player_id.clone());
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
    ws.on_upgrade(|socket| handle_socket(socket, state, player_id))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<Mutex<WebsocketGame>>, player_id: String) {
    println!("bck | New bot connecting...");

    // for each bot, create two channels
    // 1. to send messages containing the actions received from the client-bot over the websocket to the server-bot from our logic (from action_sender to action_receiver)
    // 2. to send the game state and possible actions that that are called from the server on the bot (from state_sender to state_receiver)
    // to the websocket so it can send it to the client-bot

    println!("bck | Bot connected with ID: {}", player_id.clone());
    let (mut state_receiver, action_sender) = state
        .lock()
        .await
        .channel_per_player
        .lock()
        .await
        .remove(&player_id)
        .unwrap();

    state
        .lock()
        .await
        .connected_players
        .lock()
        .await
        .insert(player_id.clone(), true);

    loop {
        // send state and possible actions to client-bot

        println!(
            "bck | waiting to receive game state info for bot {}",
            player_id
        );
        let recv_state: Option<Message> = state_receiver.recv().await;
        println!("bck | received game state info for bot {}", player_id);

        println!(
            "bck | game state info for bot {}: {}",
            player_id,
            recv_state.clone().unwrap().to_text().unwrap()
        );

        println!("bck | waiting to send state to client of bot {}", player_id);
        match recv_state {
            Some(msg) => {
                if let Err(e) = socket.send(msg).await {
                    eprintln!("bck | Error sending message: {}", e);
                }
            }
            None => todo!(),
        }
        println!(
            "bck | finished sending state to client of bot {}",
            player_id
        );

        // receive action and send to server-bot
        println!(
            "bck | waiting to receive action from client of bot {}",
            player_id
        );
        if let Some(Ok(msg)) = socket.recv().await {
            println!(
                "bck | bot {} action: {}",
                player_id,
                msg.clone().to_text().unwrap()
            );
            println!(
                "bck | finished receiving action from client of bot {}",
                player_id
            );

            println!("bck | waiting to send action of bot {} to game", player_id);
            match msg {
                Message::Text(_) => match action_sender.send(msg).await {
                    Ok(_) => println!("bck | action of bot {} has been send to game", player_id),
                    Err(_) => {
                        eprintln!("bck | failure sending action of bot {} to game", player_id)
                    }
                },
                Message::Close(_) => {
                    println!("bck | Closing WebSocket connection of bot {}.", player_id);
                    break;
                }
                _ => {
                    eprint!(
                        "received unknown message type from client of bot {}, closing WebSocket connection",
                        player_id
                    );
                    break;
                }
            }
        }
    }

    println!("bck | Bot ID {} disconnected.", player_id);

    state
        .lock()
        .await
        .connected_players
        .lock()
        .await
        .insert(player_id, false);

    state_receiver.close();
}

pub async fn organize_new_game(state: Arc<Mutex<WebsocketGame>>) {
    let mut missing_players = state.lock().await.get_missing_players().await;
    while missing_players.len() > 0 {
        println!("bck | Waiting for missing players: {:?}", missing_players);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        missing_players = state.lock().await.get_missing_players().await;
    }

    println!("bck | all players joined");
    loop {
        missing_players = state.lock().await.get_missing_players().await;
        if missing_players.len() > 0 {
            println!(
                "The game should be interrupted and the following players removed: {:?}",
                missing_players
            );
        }

        println!("bck | It's bot ID ___'s turn.",);
        // todo, should i use this: state.game.play_one_round();

        // todo should we send the game state for each round to each player,
        // effectively no difference but then the player could have more time
        // to calculate stuff while other player calculate their actions, but could also be more complicated
        // to implement the bot this way

        // Wait for a short period to allow the bot to respond.
        // In a real game, you would implement a more robust system for
        // waiting for responses and handling timeouts.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
