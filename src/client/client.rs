use crate::client::my_bot::MyBot;
use crate::messages::message_protocol::{ActionMessage, StateMessage};
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use axum::extract::ws::Utf8Bytes;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

#[tokio::main]
async fn main() {
    let mut my_bot = MyBot {};
    let (ws_stream, _) = connect_async("ws://127.0.0.1:3000/game?player_id=ufuk")
        .await
        .expect("Failed to connect");
    println!("Connected to server!");

    let (mut send, mut recv) = ws_stream.split();

    // Spawn a task to listen for incoming messages
    while let Some(msg) = recv.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let state_msg: StateMessage = serde_json::from_str(&text).unwrap();
                println!("client received message: {}", text);

                let action_msg: ActionMessage = state_msg.call_action(&mut my_bot);

                let _ = send.send(Message::Text(serde_json::to_string(&action_msg).unwrap()));
            }

            Ok(Message::Close(_)) => {
                println!("Connection closed by server");
                break;
            }
            Ok(other) => {
                println!("Received other message: {:?}", other);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}
