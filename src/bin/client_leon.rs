use axum::extract::ws::Utf8Bytes;
use futures_util::{SinkExt, StreamExt};
use kuh_handel::messages::message_protocol::{ActionMessage, StateMessage};
use kuh_handel::model::player::player_actions::base_player_actions::PlayerActions;
use kuh_handel::my_bot::MyBot;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

#[tokio::main]
async fn main() {
    let my_name = "leon".to_string();
    let mut my_bot = MyBot::new(my_name.clone());
    let (ws_stream, _) = connect_async(format!("ws://127.0.0.1:3000/game?player_id={}", my_name))
        .await
        .expect("Failed to connect");
    println!("Connected to server!");

    let (mut send, mut recv) = ws_stream.split();

    // Spawn a task to listen for incoming messages
    while let Some(msg) = recv.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let state_msg: StateMessage = serde_json::from_str(&text).unwrap();
                println!("bot {} received message: {}", my_name, text);

                let action_msg: ActionMessage = my_bot.map_to_action(state_msg);

                println!(
                    "bot {} picked action: {}",
                    my_name,
                    serde_json::to_string(&action_msg).unwrap()
                );
                let _ = send
                    .send(Message::Text(serde_json::to_string(&action_msg).unwrap()))
                    .await;
                println!("bot {}, finished sending action", my_name)
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
        println!("waiting for next action request");
    }
}
