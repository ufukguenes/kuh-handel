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
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;

use backend_api::{games_per_second_handler, websocket_handler};
use model::match_making::{WebsocketLobby, organize_new_game};

use std::{net::SocketAddr, sync::Arc};

use tokio;
use tracing_appender::non_blocking;
use tracing_subscriber::{EnvFilter, fmt};

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
    let env_filter = EnvFilter::try_from_env("KUH_LOG").unwrap_or_else(|_| EnvFilter::new("error"));

    fmt()
        .with_writer(log_writer)
        .with_ansi(false)
        .with_env_filter(env_filter)
        .finish()
        .init();
    // diable tracing of info! with:

    let multi_progress = MultiProgress::new();

    let mut authentication =
        JsonLog::<String>::new("authentication.json".to_string(), "new users".to_string());

    authentication.add_to_multi_progress(&multi_progress).await;

    match authentication.init_from_file().await {
        Ok(_) => (),
        Err(_) => authentication.to_file().await.unwrap(),
    };

    let seed: u64 = 0;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let bot_time_out = tokio::time::Duration::from_millis(500);
    let interactive_player_time_out_min = tokio::time::Duration::from_secs(5 * 60);

    let (pvp_ws_lobby, game_results, _) = create_lobby_log_handle_pair(
        10,
        "pvp_games".into(),
        rng.random(),
        (3, 6),
        3,
        false,
        true,
        bot_time_out,
        &multi_progress,
    )
    .await;

    let (server_bot_ws_lobby, server_bot_results, _) = create_lobby_log_handle_pair(
        10,
        "server_bot_games".into(),
        rng.random(),
        (3, 6),
        1,
        true,
        false,
        bot_time_out,
        &multi_progress,
    )
    .await;

    let (interactive_ws_lobby, interactive_results, _) = create_lobby_log_handle_pair(
        10,
        "interactive_games".into(),
        rng.random(),
        (3, 3),
        3,
        false,
        false,
        interactive_player_time_out_min,
        &multi_progress,
    )
    .await;

    let usize_logs = Arc::new(vec![
        game_results.clone(),
        server_bot_results.clone(),
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
            "/kuh-handel/pvp_games",
            routing::get(websocket_handler)
                .with_state((pvp_ws_lobby.clone(), authentication.clone())),
        )
        .route(
            "/kuh-handel/server_bot_game",
            routing::get(websocket_handler)
                .with_state((server_bot_ws_lobby.clone(), authentication.clone())),
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
    play_only_against_server_bot: bool,
    sync_game_starts: bool,
    time_out: tokio::time::Duration,
    multi_progress: &MultiProgress,
) -> (
    WebsocketLobby,
    JsonLog<Vec<usize>>,
    tokio::task::JoinHandle<()>,
) {
    let mut log = JsonLog::<Vec<usize>>::new(
        format!("./results/{}.json", lobby_name),
        format!("new {}", lobby_name.clone()),
    );
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
        play_only_against_server_bot,
        sync_game_starts,
    ));

    return (lobby, log, handle);
}
