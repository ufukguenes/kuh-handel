mod model {
    mod game_factory;
    pub mod game_logic;
    pub mod match_making;
}

mod backend_api;
mod game_error;
mod server_side_player;

use axum::{Router, routing};

use indicatif::MultiProgress;
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;

use backend_api::{games_per_second_handler, websocket_handler};
use model::match_making::{WebsocketLobby, organize_new_game};

use std::{net::SocketAddr, sync::Arc};

use tokio;
use tracing_appender::non_blocking;
use tracing_subscriber::fmt;

use crate::backend_api::{JsonLog, register_handler, stats_handler};

// TODO:
// - create python client/ wrapper
// - documentation
// - minimize bloated logging
// - new rankings: sum of all points, heatmap of all players against all players, squared sum of all positions of one player

#[tokio::main]
async fn main() {
    let game_log_file = tracing_appender::rolling::minutely("logs", "app.log");
    let (log_writer, _guard1) = non_blocking(game_log_file);

    fmt()
        .with_writer(log_writer)
        .with_ansi(false)
        .with_max_level(Level::ERROR)
        .finish()
        .init();
    // diable tracing of info! with: .with_max_level(Level::ERROR)

    let multi_progress = MultiProgress::new();

    let mut authentication = JsonLog::<String>::new(
        "authentication.json".to_string(),
        "registerd users".to_string(),
    );

    authentication.add_to_multi_progress(&multi_progress).await;

    match authentication.init_from_file().await {
        Ok(_) => (),
        Err(_) => authentication.to_file().await.unwrap(),
    };

    let bot_time_out = tokio::time::Duration::from_millis(500);
    let interactive_player_time_out_min = tokio::time::Duration::from_secs(5 * 60);

    let (pvp_ws_lobby, mut game_results, _) = create_lobby_log_handle_pair(
        10,
        "game_results".into(),
        0,
        (3, 6),
        3,
        false,
        bot_time_out,
        &multi_progress,
    )
    .await;

    let (random_ws_lobby, mut random_results, _) = create_lobby_log_handle_pair(
        10,
        "random_results".into(),
        0,
        (3, 6),
        1,
        true,
        bot_time_out,
        &multi_progress,
    )
    .await;

    let (interactive_ws_lobby, mut interactive_results, _) = create_lobby_log_handle_pair(
        10,
        "interactive_results".into(),
        0,
        (3, 6),
        2,
        false,
        interactive_player_time_out_min,
        &multi_progress,
    )
    .await;

    let usize_logs = Arc::new(vec![
        game_results.clone(),
        random_results.clone(),
        interactive_results.clone(),
    ]);
    let auth_log = authentication.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        interval.tick().await;
        loop {
            interval.tick().await;

            auth_log.update_terminal_view().await;
            for log in usize_logs.iter() {
                log.update_terminal_view().await;
            }
        }
    });

    // init websocket through http websocket upgrade
    let app: Router = Router::new()
        .route(
            "/kuh-handel/register",
            routing::post(register_handler).with_state(authentication.clone()),
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
            routing::get(websocket_handler)
                .with_state((pvp_ws_lobby.clone(), authentication.clone())),
        )
        .route(
            "/kuh-handel/random_game",
            routing::get(websocket_handler)
                .with_state((random_ws_lobby.clone(), authentication.clone())),
        )
        .route(
            "/kuh-handel/interactive_game",
            routing::get(websocket_handler)
                .with_state((interactive_ws_lobby.clone(), authentication.clone())),
        );

    let port = 2000;
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();

    println!("Started server on port {}", port);
    print!("Waiting for first game to finish");
    axum::serve(listener, app).await.unwrap();
}

pub async fn create_lobby_log_handle_pair(
    average_time_over_n_games: usize,
    lobby_name: String,
    seed: u64,
    (min_game_size, max_game_size): (usize, usize),
    min_ws_player_amount: usize,
    play_only_against_random_bots: bool,
    time_out: tokio::time::Duration,
    multi_progress: &MultiProgress,
) -> (
    WebsocketLobby,
    JsonLog<Vec<usize>>,
    tokio::task::JoinHandle<()>,
) {
    let mut log = JsonLog::<Vec<usize>>::new(format!("{}.json", lobby_name), lobby_name.clone());
    log.add_to_multi_progress(&multi_progress).await;

    match log.init_from_file().await {
        Ok(_) => (),
        Err(_) => log.to_file().await.unwrap(),
    };

    let lobby = WebsocketLobby::new_default(lobby_name, average_time_over_n_games, time_out);

    // start the game in a separate thread, so that server can handle connections
    let handle = tokio::spawn(organize_new_game(
        lobby.clone(),
        log.clone(),
        seed,
        (min_game_size, max_game_size),
        min_ws_player_amount,
        play_only_against_random_bots,
    ));

    return (lobby, log, handle);
}
