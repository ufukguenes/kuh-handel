use crate::client::MyBot::MyBot;

use crate::model::player::player_actions::base_player_actions::PlayerActions;
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

    let (mut write, mut read) = ws_stream.split();

    // Spawn a task to listen for incoming messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received: {}", text);
                let which_action: bool;

                match which_action {
                    True => {
                        let action = my_bot.draw_or_trade();
                        write.send(action).await.unwrap();
                    }

                    False => {
                        my_bot.receive_game_update(update);
                    }
                }
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
