mod backend_api;
mod model;

use axum::{Router, routing};
use model::animals::AnimalSet;
use model::animals::{AnimalSetFactory, DefaultAnimalSetFactory};
use model::game_logic::Game;
use std::sync::Arc;

use backend_api::{WebsocketGame, WebsocketPlayer, organize_new_game, websocket_handler};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let animal_set: AnimalSet = DefaultAnimalSetFactory::new(500, vec![0, 4]);
    let seed: u64 = 0;
    let game: Game<model::player::RandomPlayerActions> = Game::new_random_game(
        vec![
            String::from("ufuk"),
            String::from("leon"),
            String::from("gregor"),
        ],
        seed,
    );
    println!("Animal value: {}", animal_set);

    println!("{}", game);

    let ws_game = WebsocketGame::new(Arc::new(game));
    // start the game in a seperate thread, so that server can handle connections
    tokio::spawn(organize_new_game(Arc::clone(&ws_game)));

    // init websocket through http websocket upgrade
    let app: Router =
        Router::new().route("/game", routing::get(websocket_handler).with_state(ws_game));
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
