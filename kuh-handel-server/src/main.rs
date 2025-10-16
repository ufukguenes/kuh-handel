pub mod backend_api;
pub mod model;
pub mod server_side_player;

use axum::{Router, routing};

use std::sync::Arc;

use tokio::sync::Mutex;

use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::util::SubscriberInitExt;

use backend_api::{WebsocketLobby, organize_new_game, websocket_handler};

use std::net::SocketAddr;

use tokio;
use tracing_appender::non_blocking;
use tracing_subscriber::fmt;

// TODO:
// - real matchmaking
// - should we remove money and value and make them type renames?
// - restructure libraries to be able to publish
// - create python client/ wrapper
// - remove dangerous unwraps, ?, etc...
// - we might not need AnimalSet, consider removing that
// - store password and player_ids and check for duplicate ids
// - provide demo, where players can test their bot against our random bots for testing
// - maybe also provide test people can make so that they can see what goes wrong? (actually we have that already, we have the supervisor, who checks if a move is valid)
// - make a simple visualization for that that is also hosted on the website
// - should we allow for one bot to play multiple games in parallel, or just one game per bot at any time?

#[tokio::main]
async fn main() {
    let game_log_file = tracing_appender::rolling::never("logs", "app.log");
    let (log_writer, _guard1) = non_blocking(game_log_file);

    let game_results_file = tracing_appender::rolling::never("logs", "results.log");
    let (results_writer, _guard2) = non_blocking(game_results_file);

    fmt()
        .with_writer(log_writer.and(results_writer.with_filter(|meta| meta.target() == "results")))
        .with_ansi(false)
        .finish()
        .init();

    let ws_lobby = Arc::new(Mutex::new(WebsocketLobby::new()));
    // start the game in a separate thread, so that server can handle connections
    tokio::spawn(organize_new_game(Arc::clone(&ws_lobby)));

    // init websocket through http websocket upgrade
    let app: Router = Router::new().route(
        "/game",
        routing::get(websocket_handler).with_state(ws_lobby),
    );
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
