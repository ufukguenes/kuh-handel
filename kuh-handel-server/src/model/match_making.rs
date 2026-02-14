use crate::backend_api::JsonLog;
use crate::model::game_logic::Game;
use crate::server_side_player::websocket_actions::WebsocketActions;

use kuh_handel_lib::player::base_player::PlayerId;
use kuh_handel_lib::player::player_actions::PlayerActions;
use kuh_handel_lib::player::simple_player::SimplePlayer;

use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::cmp::min;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinError;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tracing::{Level, error, info};

#[derive(Clone)]
pub struct WebsocketLobby {
    pub lobby_name: String,
    pub channels_for_ws_actions: Arc<
        Mutex<BTreeMap<PlayerId, Option<(Sender<serde_json::Value>, Receiver<serde_json::Value>)>>>,
    >,
    pub time_last_n_games: Arc<Mutex<Vec<tokio::time::Duration>>>,
    pub average_time_over_n_games: usize,
    pub player_time_out: tokio::time::Duration,
}

impl WebsocketLobby {
    pub fn new_default(
        lobby_name: String,
        average_time_over_n_games: usize,
        player_time_out: tokio::time::Duration,
    ) -> Self {
        WebsocketLobby {
            lobby_name,
            channels_for_ws_actions: Arc::default(),
            time_last_n_games: Arc::default(),
            average_time_over_n_games: average_time_over_n_games,
            player_time_out,
        }
    }

    pub async fn games_per_second(&self) -> f32 {
        let locked_times = self.time_last_n_games.lock().await;
        let summed_times = locked_times.iter().sum::<Duration>().as_secs_f32();
        locked_times.len() as f32 / summed_times
    }
}

pub async fn organize_new_game(
    ws_lobby: WebsocketLobby,
    game_results: JsonLog<Vec<usize>>,
    seed: u64,
    (min_game_size, max_game_size): (usize, usize),
    min_ws_player_amount: usize,
    play_only_against_server_bot: bool,
    sync_game_starts: bool,
) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let valid_game_sizes: Vec<usize> = (min_game_size..=max_game_size).collect();

    loop {
        info!("og | creating new round of games {}", ws_lobby.lobby_name);

        let mut current_game_handles = Vec::new();

        let mut available_players_ids = Vec::new();
        {
            let channels_for_ws_actions = ws_lobby.channels_for_ws_actions.lock().await;
            for (id, channels) in channels_for_ws_actions.iter() {
                if channels.is_some() {
                    available_players_ids.push(id.clone());
                }
            }
        }

        let num_players = available_players_ids.len();

        if min_ws_player_amount > num_players {
            info!(
                "og | not enough players have joined, waiting {}",
                ws_lobby.lobby_name
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            continue;
        }

        available_players_ids.shuffle(&mut rng);

        let players_per_game;

        if play_only_against_server_bot {
            players_per_game = 1;
        } else {
            let remainders: Vec<usize> = valid_game_sizes.iter().map(|i| num_players % i).collect();

            let (min_index, &min_value) = remainders
                .iter()
                .enumerate()
                .min_by(|&(_, a), &(_, b)| a.cmp(b))
                .unwrap();
            let (max_index, _) = remainders
                .iter()
                .enumerate()
                .max_by(|&(_, a), &(_, b)| a.cmp(b))
                .unwrap();

            if min_value == 0 {
                players_per_game = *valid_game_sizes.get(min_index).unwrap();
            } else {
                players_per_game = *valid_game_sizes.get(max_index).unwrap();
            }
        }

        for _ in (0..num_players).step_by(players_per_game) {
            let last_value = min(players_per_game, available_players_ids.len());
            let current_players: Vec<String> = available_players_ids.drain(0..last_value).collect();

            let new_ws_lobby = ws_lobby.clone();

            let num_current_players = current_players.len();

            let min_server_bots = min_game_size.checked_sub(num_current_players).unwrap_or(0);
            let max_server_bots = max_game_size.checked_sub(num_current_players).unwrap_or(0);

            let num_server_bots = rng.random_range(min_server_bots..=max_server_bots);

            let server_bots: Vec<String> = (0..num_server_bots)
                .map(|i| String::from(format!("server_bot_{}", i)))
                .collect();
            info!(
                "og| {} create game with {:?}",
                ws_lobby.lobby_name, current_players
            );
            let game_handle =
                spawn_game(new_ws_lobby.clone(), current_players, server_bots, &mut rng);

            let cloned_game_results = game_results.clone();

            let handle = tokio::spawn(async move {
                let ranking = game_handle.await;
                update_results(cloned_game_results.clone(), &ranking).await;
                cloned_game_results.increase_count().await;
            });

            current_game_handles.push(handle);
        }

        if sync_game_starts {
            for handle in current_game_handles {
                let _ = handle.await;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

async fn update_results(
    game_results: JsonLog<Vec<usize>>,
    ranking: &Result<Vec<(PlayerId, usize)>, JoinError>,
) {
    match ranking {
        Ok(ranking) => {
            let mut result_map = game_results.data.lock().await;
            for (rank, (player, _points)) in ranking.iter().enumerate() {
                result_map
                    .entry(player.clone())
                    .or_insert(Vec::new())
                    .push(rank);
            }
        }
        Err(_) => error!("og | game not properly finished"),
    }

    let _ = game_results.to_file().await;
}

pub fn spawn_game(
    ws_lobby: WebsocketLobby,
    ws_players: Vec<String>,
    mut server_bots: Vec<String>,
    rng: &mut ChaCha8Rng,
) -> JoinHandle<Vec<(PlayerId, usize)>> {
    let amount_seeds = ws_players.len() + server_bots.len() + 1;
    let seeds: Vec<u64> = rng.random_iter().take(amount_seeds).collect();
    tokio::spawn(async move {
        let start_time = tokio::time::Instant::now();

        let mut ws_actions: Vec<WebsocketActions> = Vec::new();

        {
            let mut channel_for_ws_actions = ws_lobby.channels_for_ws_actions.lock().await;

            for id in &ws_players {
                let player_channels = channel_for_ws_actions.get_mut(id).unwrap();
                ws_actions.push(WebsocketActions::new(
                    id.clone(),
                    player_channels.take().unwrap(),
                    *seeds.first().unwrap(),
                ));
            }
        }

        let mut server_bot_actions: Vec<SimplePlayer> = Vec::new();
        for idx in 0..server_bots.len() {
            let aggressiveness = SimplePlayer::get_random_aggressiveness(*seeds.first().unwrap());
            let old_id = server_bots[idx].clone();
            let new_id = format!("{old_id}_aggressiveness_{aggressiveness}");
            server_bots[idx] = new_id.clone();
            server_bot_actions.push(SimplePlayer::new(new_id.clone(), aggressiveness));
        }

        let mut all_ids: Vec<String> = Vec::new();
        all_ids.extend(ws_players.clone());
        all_ids.extend(server_bots.clone());

        let game_handle = tokio::task::spawn_blocking(move || {
            let mut all_actions: Vec<Box<dyn PlayerActions>> = Vec::new();
            all_actions.extend(
                ws_actions
                    .into_iter()
                    .map(|action: WebsocketActions| Box::new(action) as Box<dyn PlayerActions>),
            );
            all_actions.extend(
                server_bot_actions
                    .into_iter()
                    .map(|action: SimplePlayer| Box::new(action) as Box<dyn PlayerActions>),
            );

            let mut game = Game::new_default_game(all_ids, all_actions, *seeds.first().unwrap());

            let ranking = game.play();

            ranking
        });
        let ranking = game_handle.await.unwrap();
        match ranking {
            Ok(ranking) => {
                let mut timed_games = ws_lobby.time_last_n_games.lock().await;
                if timed_games.len() == ws_lobby.average_time_over_n_games {
                    timed_games.remove(0);
                }
                let time_diff = tokio::time::Instant::now().duration_since(start_time);
                timed_games.push(time_diff);

                tracing::event!(target: "ranking", Level::INFO, "{:?}", ranking);
                ranking
            }
            Err(err) => {
                println!("{:?}", err);
                Vec::new()
            }
        }
    })
}
