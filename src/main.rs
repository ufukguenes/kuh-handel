use axum::extract::ws::Message;
use axum::{Router, routing};
use kuh_handel::model::animals::{AnimalSet, AnimalSetFactory, DefaultAnimalSetFactory};
use kuh_handel::model::game_logic::Game;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::vec;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::util::SubscriberInitExt;

use kuh_handel::backend_api::{WebsocketGame, organize_new_game, websocket_handler};
use kuh_handel::model::player::player_actions::websocket_actions::WebsocketActions;
use std::net::SocketAddr;

use kuh_handel::model::player::player_actions::random_actions::RandomPlayerActions;

use tokio;
use tracing_appender::non_blocking;
use tracing_subscriber::fmt;

// TODO:
// - calculate the winner of a game and log that in a file
// - create matches in multiple threads and handle who plays against who
// - we might not need AnimalSet, consider removing that
// - add an authentication key for each player so others cant just copy the name
// - provide demo, where players can test their bot against our random bots for testing
// - maybe also provide test people can make so that they can see what goes wrong? (actually we have that already, we have the supervisor, who checks if a move is valid)
// - make a simple visualization for that that is also hosted on the website

#[tokio::main]
async fn main() {
    let game_log_file = tracing_appender::rolling::never("logs", "app.log");
    let (log_writer, _guard1) = non_blocking(game_log_file);

    let game_results_file = tracing_appender::rolling::never("logs", "results.log");
    let (results_writer, _guard2) = non_blocking(game_results_file);

    fmt()
        .with_writer(log_writer.and(results_writer.with_filter(|meta| meta.target() == "winner")))
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

        let results = game.play().unwrap();

        println!("ranking: {:?}", results);

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
