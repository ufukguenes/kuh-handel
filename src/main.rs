use axum::extract::ws::Message;
use axum::{Router, routing};
use kuh_handel::model::animals::{AnimalSet, AnimalSetFactory, DefaultAnimalSetFactory};
use kuh_handel::model::game_logic::Game;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::vec;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing_subscriber::util::SubscriberInitExt;

use kuh_handel::backend_api::{WebsocketGame, organize_new_game, websocket_handler};
use kuh_handel::model::player::player_actions::websocket_actions::WebsocketActions;
use std::net::SocketAddr;

use kuh_handel::model::player::player_actions::random_actions::RandomPlayerActions;

use tokio;
use tracing_appender::non_blocking;
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() {
    let file_appender = tracing_appender::rolling::never("logs", "app.log");
    let (non_blocking, _guard) = non_blocking(file_appender);

    fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .finish()
        .init();

    let animal_set: AnimalSet = DefaultAnimalSetFactory::new(500, vec![0, 4]);
    let (ufuk_ws_action, ufuk_channel) = WebsocketActions::new("ufuk".to_string());
    let (leon_ws_action, leon_channel) = WebsocketActions::new("leon".to_string());
    let gregor_random_action = RandomPlayerActions::new("gregor".to_string(), 25);
    let seed: u64 = 0;
    let game_handle = tokio::task::spawn_blocking(move || {
        println!("-------Default game--------\n");
        let mut game = Game::new_default_game(
            vec![
                String::from("ufuk"),
                String::from("leon"),
                String::from("gregor"),
            ],
            vec![
                Box::new(ufuk_ws_action),
                Box::new(leon_ws_action),
                Box::new(gregor_random_action),
            ],
            seed,
        );

        game.num_players();
        println!("{}", game);

        game.num_players();

        game.play().unwrap();
        println!("{}", game);

        print!("game is done")
    });

    let websocket_channels_per_player: BTreeMap<String, (Receiver<Message>, Sender<Message>)> =
        BTreeMap::from([
            ("ufuk".to_string(), ufuk_channel),
            ("leon".to_string(), leon_channel),
        ]);

    let ws_game = Arc::new(Mutex::new(
        WebsocketGame::new(Arc::new(Mutex::new(websocket_channels_per_player)))
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
