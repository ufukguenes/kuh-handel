use crate::model::{
    self,
    game_errors::GameError,
    player::{
        base_player::Player,
        player_actions::{base_player_actions::PlayerActions, websocket_actions::WebsocketActions},
    },
};

use axum::{
    extract::{
        State,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    hash::Hash,
    rc::Rc,
};
use tokio::sync::Mutex;

use tokio::sync::mpsc;

use model::game_logic::Game;

// Define the game state. It now tracks players and the current turn.
pub struct WebsocketGame<'a, T>
where
    T: PlayerActions,
{
    game: Arc<Game<T>>,
    connected_players: HashMap<String, (&'a mpsc::Receiver<Message>, &'a mpsc::Sender<Message>)>,
    websocket_players: Vec<&'a Player<WebsocketActions>>,
}

impl<'a, T> WebsocketGame<'a, T>
where
    T: PlayerActions,
{
    pub fn new(
        game: Arc<Game<T>>,
        websocket_players: Vec<&'a Player<WebsocketActions>>,
    ) -> Result<WebsocketGame<'a, T>, GameError> {
        let mut connected_players: HashMap<
            String,
            (&mpsc::Receiver<Message>, &mpsc::Sender<Message>),
        > = HashMap::new();
        let all_player_ids = game.get_all_ids();

        let mut player_id;
        let mut channels;
        for ws_player in websocket_players.iter() {
            player_id = ws_player.id().to_string();
            channels = ws_player.player_actions.get_channels();

            if all_player_ids.contains(&player_id) {
                connected_players.insert(player_id, channels);
            } else {
                return Result::Err(GameError::PlayerNotFound);
            }
        }

        Result::Ok(WebsocketGame {
            game: game,
            connected_players: connected_players,
            websocket_players: websocket_players,
        })
    }

    pub fn get_missing_players(&self) -> Vec<String> {
        let mut missing_players: Vec<String> = Vec::new();
        for ws_player in self.websocket_players.iter() {
            if !self.connected_players.contains_key(ws_player.id()) {
                missing_players.push(ws_player.id().to_string());
            }
        }

        return missing_players;
    }

    pub fn insert_channels(
        &mut self,
        player_id: String,
    ) -> Result<(&'a mpsc::Receiver<Message>, &'a mpsc::Sender<Message>), GameError> {
        match self
            .websocket_players
            .iter()
            .find(|player| player.id() == player_id)
        {
            Some(player) => {
                let new_channels = player.player_actions.get_channels();
                self.connected_players
                    .insert(player_id.clone(), new_channels);
                return Result::Ok(new_channels);
            }
            None => return Result::Err(GameError::PlayerNotFound),
        };
    }

    pub fn delete_channels(&mut self, player_id: String) {
        // todo  close the connection, but doing that gives borrow issues, that i don't want to deal with now
        self.connected_players.remove(&player_id);
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

pub async fn websocket_handler<'a, T>(
    ws: WebSocketUpgrade,
    State(state): State<WebsocketGame<'a, T>>,
) -> impl IntoResponse
where
    T: PlayerActions,
{
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket<'a, T>(socket: WebSocket, state: WebsocketGame<'a, T>)
where
    T: PlayerActions,
{
    let player_id = todo!("player id i need to parse form request");
    println!("New bot connecting...");

    if let Result::Ok((state_receiver, action_sender)) = state.insert_channels(player_id) {
        // for each bot, create two channels
        // 1. to send messages containing the actions received from the client-bot over the websocket to the server-bot from our logic (from action_sender to action_receiver)
        // 2. to send the game state and possible actions that that are called from the server on the bot (from state_sender to state_receiver)
        // to the websocket so it can send it to the client-bot

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

        state.delete_channels(player_id);
    } else {
        println!("Connection failed for bot with ID: {}", player_id)
    };
}

pub async fn organize_new_game<T>(state: WebsocketGame<'_, T>)
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
            todo!("wait for possible reconnect");
            print!("removing players: {:?}", missing_players);
            for player_id in missing_players {
                state.game.remove_player(player_id);
            }
        }

        let current_player = state.game.get_player_for_current_turn();
        println!("\nIt's bot ID {}'s turn.", current_player.borrow());
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
