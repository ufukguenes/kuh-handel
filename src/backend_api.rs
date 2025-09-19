use crate::model::{
    self,
    player::{self, Player, PlayerActions},
};

use axum::{
    extract::{
        State,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use model::game_logic::Game;

// Define the game state. It now tracks players and the current turn.
pub struct WebsocketGame<T>
where
    T: PlayerActions,
{
    connected_players: Vec<WebsocketPlayer<T>>,
    game: Arc<Game<T>>,
}

impl<T> WebsocketGame<T>
where
    T: PlayerActions,
{
    pub fn new(game: Arc<Game<T>>) -> Self {
        WebsocketGame {
            connected_players: Vec::new(),
            game: game,
        }
    }

    pub fn get_sender_for_player(&self, player: &Player<T>) -> Option<mpsc::Sender<Message>> {
        // todo can i clone the sender without breaking the connection, or should i borrow with Arc?
        self.connected_players
            .iter()
            .find(|p| p.player.get_string_id() == player.get_string_id())
            .map(|p| p.sender.clone())
    }

    pub fn get_missing_players(&self) -> Vec<String> {
        let game: Arc<Game<T>> = Arc::clone(&self.game);
        let connected_players = &self.connected_players;

        game.get_all_ids()
            .iter()
            .filter(|id| {
                !connected_players
                    .iter()
                    .any(|p| &&p.player.get_string_id() == id)
            })
            .map(|str| str.clone())
            .collect()
    }
}

// todo this should just be a hash map inside the websocket game
pub struct WebsocketPlayer<T>
where
    T: PlayerActions,
{
    player: Arc<Player<T>>,
    sender: mpsc::Sender<Message>,
}

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
    println!("New bot connecting...");

    // Create a channel to send messages from the game loop to this specific bot.
    let (tx, mut rx) = mpsc::channel(1);

    let player_id = String::from("player id i need to parse form request");
    let new_websocket_player = WebsocketPlayer {
        player: state.game.get_player_by_id(player_id),
        sender: tx,
    };

    println!("Bot connected with ID: {}", player_id);

    // Split the WebSocket into a sender and a receiver.
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Use a separate task to handle messages from the game loop.
    //todo do i need this?
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // The main loop for receiving messages from the bot.
    while let Some(msg) = ws_receiver.next().await {
        if let Ok(Message::Text(text)) = msg {
            // Here, you would process the bot's action based on the game state.
            println!("Received action from bot ID {}: {}", player_id, text.trim());
        }
    }

    println!("Bot ID {} disconnected.", player_id);

    // Clean up on disconnection.
    state
        .connected_players
        .retain(|player| player.player.get_string_id() != player_id);
}

async fn organize_new_game<T>(state: WebsocketGame<T>)
where
    T: PlayerActions,
{
    let mut missing_players = state.get_missing_players();
    loop {
        if missing_players.len() == 0 {
            break;
        }
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
        let current_sender = state.get_sender_for_player(current_player).unwrap();

        println!("\nIt's bot ID {}'s turn.", current_player);

        // Send a turn message to the current bot.
        let turn_message = format!(
            "Your turn, bot ID {}. Please provide an action.\n",
            current_player
        );

        if let Err(e) = current_sender
            .send(Message::Text(Utf8Bytes::from(turn_message)))
            .await
        {
            eprintln!(
                "Failed to send turn message to bot ID {}: {}",
                current_player, e
            );
        }

        // Wait for a short period to allow the bot to respond.
        // In a real game, you would implement a more robust system for
        // waiting for responses and handling timeouts.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        todo!("do game step")
    }
}
