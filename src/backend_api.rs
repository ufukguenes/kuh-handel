use crate::model::{
    self, player::base_player::Player, player::player_actions::base_player_actions::PlayerActions,
};

use axum::{
    extract::{
        State,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    hash::Hash,
    rc::Rc,
};
use std::{intrinsics::breakpoint, sync::Arc};
use tokio::sync::Mutex;

use tokio::sync::mpsc;

use model::game_logic::Game;

// Define the game state. It now tracks players and the current turn.
pub struct WebsocketGame<T>
where
    T: PlayerActions,
{
    game: Arc<Game<T>>,
}

impl<T> WebsocketGame<T>
where
    T: PlayerActions,
{
    pub fn get_missing_players(&self) -> Vec<String> {
        self.game
            .get_all_ids()
            .iter()
            .filter(|id| !self.connected_players.keys().any(|key_id| &key_id == id))
            .map(|str| str.clone())
            .collect()
    }
}

// todo this should just be a hash map inside the websocket game

/*
#[tokio::main]
async fn main() {
    // The server's game state.
    let game_state = Arc::new(GameState {
        players: Mutex::new(VecDeque::new()),
        turn_index: Mutex::new(0),
    });

    // Start the game loop in a separate, dedicated task.
    // This allows the server to handle new connections and the game logic concurrently.
    tokio::spawn(organize_new_game(Arc::clone(&game_state)));

    // Configure the Axum router to handle a single WebSocket route.
    let app = Router::new()
        .route("/game", get(ws_handler))
        .with_state(game_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 7878));
    println!("Server is listening on {}", addr);

    // Start the server and serve the application.
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
    */

pub async fn websocket_handler<T>(
    ws: WebSocketUpgrade,
    State(state): State<WebsocketGame<T>>,
) -> impl IntoResponse
where
    T: PlayerActions,
{
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket<'a, T>(socket: WebSocket, state: WebsocketGame<T>)
where
    T: PlayerActions,
{
    let player_id = todo!("player id i need to parse form request");
    println!("New bot connecting...");

    // for each bot, create two channels
    // 1. to send messages containing the actions received from the client-bot over the websocket to the server-bot from our logic (from action_sender to action_receiver)
    // 2. to send the game state and possible actions that that are called from the server on the bot (from state_sender to state_receiver)
    // to the websocket so it can send it to the client-bot

    let (state_sender, state_receiver) = mpsc::channel(1);
    let (action_sender, action_receiver) = mpsc::channel(1);

    println!("Bot connected with ID: {}", player_id);

    loop {
        // send state and possible actions to client-bot
        let recv_state: Option<Message> = state_receiver.recv().await;
        match recv_state {
            Some(msg) => {
                if let Err(e) = socket.send(msg).await {
                    eprintln!("Error sending message: {}", e);
                }
            }
            None => todo!(),
        }

        // receive action and send to server-bot
        if let Some(Ok(msg)) = socket.recv().await {
            match msg {
                Message::Text(utf8_bytes) => match action_sender.send(msg).await {
                    Ok(_) => println!("action has been send to game"),
                    Err(_) => eprintln!("failure sending action to game"),
                },
                Message::Close(close_frame) => {
                    println!("Closing WebSocket connection.");
                    break;
                }
                _ => {
                    eprint!(
                        "received unknown message type from client, closing WebSocket connection"
                    );
                    break;
                }
            }
        }
    }

    println!("Bot ID {} disconnected.", player_id);

    state_receiver.close();
    action_receiver.close();
}

pub async fn organize_new_game<T>(state: WebsocketGame<T>)
where
    T: PlayerActions,
{
    let mut missing_players = state.get_missing_players();
    while missing_players.len() > 0 {
        println!("Waiting for missing players: {:?}", missing_players);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        missing_players = state.get_missing_players();
    }

    loop {
        missing_players = state.get_missing_players();
        if missing_players.len() > 0 {
            todo!("handle missing player");
        }

        let current_player = state.game.get_player_for_current_turn();
        let current_sender = state
            .connected_players
            .get(current_player.borrow().id())
            .unwrap();

        println!("\nIt's bot ID {}'s turn.", current_player.borrow());

        // Send a turn message to the current bot.
        let turn_message = format!(
            "Your turn, bot ID {}. Please provide an action.\n",
            current_player.borrow()
        );

        if let Err(e) = current_sender
            .send(Message::Text(Utf8Bytes::from(turn_message)))
            .await
        {
            eprintln!(
                "Failed to send turn message to bot ID {}: {}",
                current_player.borrow(),
                e
            );
        }

        // Wait for a short period to allow the bot to respond.
        // In a real game, you would implement a more robust system for
        // waiting for responses and handling timeouts.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
