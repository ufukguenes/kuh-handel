mod model {
    mod game_factory;
    pub mod game_logic;
    pub mod match_making;
}

mod backend_api;
mod game_error;
mod server_side_player;

use axum::{Router, routing};

use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;

use backend_api::{games_per_second_handler, pvp_websocket_handler, random_websocket_handler};
use model::match_making::{WebsocketLobby, organize_new_game};

use std::net::SocketAddr;

use tokio;
use tracing_appender::non_blocking;
use tracing_subscriber::fmt;

use crate::{
    backend_api::{JsonLog, register_handler, stats_handler},
    model::match_making::organize_random_game,
};

// TODO:
// - should we remove money and value and make them type renames?
// - create python client/ wrapper
// - remove dangerous unwraps, ?, etc...
// - we might not need AnimalSet, consider removing that
// - maybe also provide test people can make so that they can see what goes wrong? (actually we have that already, we have the supervisor, who checks if a move is valid)
// - documentation
// - minimize bloated logging

#[tokio::main]
async fn main() {
    let game_log_file = tracing_appender::rolling::minutely("logs", "app.log");
    let (log_writer, _guard1) = non_blocking(game_log_file);

    fmt()
        .with_writer(log_writer)
        .with_ansi(false)
        .finish()
        .init();
    // diable tracing of info! with: .with_max_level(Level::ERROR)

    let authentication = match JsonLog::<String>::from_file("authentication.json".to_string()).await
    {
        Ok(authentication) => authentication,
        Err(_) => JsonLog::new("authentication.json".to_string())
            .await
            .unwrap(),
    };

    let game_results = match JsonLog::<Vec<usize>>::from_file("game_results.json".to_string()).await
    {
        Ok(game_results) => game_results,
        Err(_) => JsonLog::new("game_results.json".to_string()).await.unwrap(),
    };

    let pvp_ws_lobby = WebsocketLobby::new_default(10);
    let random_ws_lobby = WebsocketLobby::new_default(10);
    // start the game in a separate thread, so that server can handle connections
    tokio::spawn(organize_new_game(
        pvp_ws_lobby.clone(),
        game_results.clone(),
        0,
        (3, 6),
    ));
    tokio::spawn(organize_random_game(
        random_ws_lobby.clone(),
        game_results.clone(),
        0,
        (3, 6),
    ));

    // init websocket through http websocket upgrade
    let app: Router = Router::new()
        .route(
            "/kuh-handel/register",
            routing::post(register_handler).with_state(authentication.clone()),
        )
        .route(
            "/kuh-handel/results",
            routing::get(|| async {
                axum::response::Html(tokio::fs::read_to_string("results.html").await.unwrap())
            }),
        )
        .route(
            "/kuh-handel/get_results",
            routing::get(stats_handler).with_state(game_results.clone()),
        )
        .route(
            "/kuh-handel/games_per_second",
            routing::get(games_per_second_handler).with_state(pvp_ws_lobby.clone()),
        )
        .route(
            "/kuh-handel/game",
            routing::get(pvp_websocket_handler)
                .with_state((pvp_ws_lobby.clone(), authentication.clone())),
        )
        .route(
            "/kuh-handel/random_game",
            routing::get(random_websocket_handler)
                .with_state((random_ws_lobby.clone(), authentication.clone())),
        );
    let address = SocketAddr::from(([127, 0, 0, 1], 2000));
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
