mod backend_api;
mod model;

use axum::{Router, routing};
use model::animals::AnimalSet;
use model::animals::{AnimalSetFactory, DefaultAnimalSetFactory};
use model::game_logic::Game;
use std::sync::Arc;
use tokio::sync::Mutex;

use backend_api::{WebsocketGame, organize_new_game, websocket_handler};
use model::player::player_actions::websocket_actions::WebsocketActions;
use std::net::SocketAddr;

use crate::model::player::player_actions;
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use crate::model::player::player_actions::random_actions::RandomPlayerActions;

#[tokio::main]
async fn main() {
    let animal_set: AnimalSet = DefaultAnimalSetFactory::new(500, vec![0, 4]);
    let (ufuk_ws_action, ufuk_channel) = WebsocketActions::new();
    let (gregor_ws_action, gregor_channel) = WebsocketActions::new();
    let (leon_ws_action, leon_channel) = WebsocketActions::new();
    let seed: u64 = 0;

    println!("-------Default game--------\n");
    let mut game = Game::new_default_game(
        vec![
            String::from("ufuk"),
            String::from("leon"),
            String::from("gregor"),
        ],
        vec![ufuk_ws_action, leon_ws_action, gregor_ws_action],
        seed,
    );

    game.num_players().await;
    println!("{}", game);

    game.num_players().await;
    //game.play().unwrap();
    println!("{}", game);

    let mut websocket_channels_per_player = vec![ufuk_channel, leon_channel, gregor_channel];

    let ws_game = Arc::new(Mutex::new(
        WebsocketGame::new(
            Arc::new(Mutex::new(game)),
            &mut websocket_channels_per_player,
        )
        .await
        .unwrap(),
    ));
    // start the game in a seperate thread, so that server can handle connections
    tokio::spawn(organize_new_game(Arc::clone(&ws_game)));

    // init websocket through http websocket upgrade
    let app: Router =
        Router::new().route("/game", routing::get(websocket_handler).with_state(ws_game));
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
