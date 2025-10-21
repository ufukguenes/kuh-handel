use crate::backend_api::JsonLog;
use crate::model::game_logic::Game;
use crate::server_side_player::websocket_actions::WebsocketActions;

use kuh_handel_lib::player::base_player::PlayerId;
use kuh_handel_lib::player::player_actions::PlayerActions;
use kuh_handel_lib::player::random_player::RandomPlayerActions;

use axum::extract::ws::Message;
pub use axum_macros::debug_handler;

use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::BTreeMap;
use std::ops::Div;
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
    pub channels_for_ws_actions:
        Arc<Mutex<BTreeMap<String, (Sender<Message>, Arc<Mutex<Receiver<Message>>>)>>>,
    pub time_last_n_games: Arc<Mutex<Vec<tokio::time::Duration>>>,
    pub average_time_over_n_games: usize,
}

impl WebsocketLobby {
    pub fn new_default(average_time_over_n_games: usize) -> Self {
        WebsocketLobby {
            channels_for_ws_actions: Arc::new(Mutex::new(BTreeMap::new())),
            time_last_n_games: Arc::new(Mutex::new(Vec::new())),
            average_time_over_n_games: average_time_over_n_games,
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
) {
    info!("og | enough players joined");
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let valid_game_sizes: Vec<usize> = (min_game_size..=max_game_size).collect();

    loop {
        if ws_lobby.channels_for_ws_actions.lock().await.len() < 3 {
            info!("og | waiting for more players to join");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        } else {
            info!("og | creating new round of games");

            // todo how to handle if player drops connection? -> just use the backup action in the websocket actions?
            let mut all_player_ids: Vec<String> = ws_lobby
                .channels_for_ws_actions
                .lock()
                .await
                .keys()
                .cloned()
                .collect();

            all_player_ids.shuffle(&mut rng);

            let num_players = all_player_ids.len();

            let remainders: Vec<usize> = valid_game_sizes.iter().map(|i| num_players % i).collect();

            let (min_index, &min_value) = remainders
                .iter()
                .enumerate()
                .min_by(|&(_, a), &(_, b)| a.cmp(b))
                .unwrap();
            let (max_index, &max_value) = remainders
                .iter()
                .enumerate()
                .max_by(|&(_, a), &(_, b)| a.cmp(b))
                .unwrap();

            let players_per_game;

            if min_value == 0 {
                players_per_game = *valid_game_sizes.get(min_index).unwrap();
            } else {
                players_per_game = *valid_game_sizes.get(max_index).unwrap();
            }

            let mut new_games = Vec::new();

            for _ in (0..num_players).step_by(players_per_game) {
                let current_players: Vec<String> = all_player_ids
                    .iter()
                    .take(players_per_game)
                    .cloned()
                    .collect();
                let new_ws_lobby = ws_lobby.clone();

                let min_random_players = min_game_size.checked_sub(players_per_game).unwrap_or(0);
                let max_random_players = max_game_size.checked_sub(players_per_game).unwrap_or(0);

                let num_random_player = rng.random_range(min_random_players..=max_random_players);

                let random_players: Vec<String> = (0..num_random_player)
                    .map(|i| String::from(format!("random_player_{}", i)))
                    .collect();

                let new_game = spawn_game(new_ws_lobby.clone(), current_players, random_players);

                new_games.push(new_game);
            }

            for game in new_games {
                let ranking = game.await;
                update_results(game_results.clone(), ranking).await
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

pub async fn organize_random_game(
    ws_lobby: WebsocketLobby,
    game_results: JsonLog<Vec<usize>>,
    seed: u64,
    (min_game_size, max_game_size): (usize, usize),
) {
    info!("og | starting to create random games");
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    loop {
        if ws_lobby.channels_for_ws_actions.lock().await.len() < 1 {
            info!("og | waiting for someone to join random game");
        } else {
            info!("og | creating new random games");

            let mut new_games = Vec::new();
            for player in ws_lobby.channels_for_ws_actions.lock().await.keys() {
                let new_ws_lobby = ws_lobby.clone();

                let num_random_players = rng.random_range(min_game_size..=max_game_size) as usize;
                let random_players: Vec<String> = (0..num_random_players)
                    .map(|i| String::from(format!("random_player_{}", i)))
                    .collect();

                let random_game =
                    spawn_game(new_ws_lobby.clone(), vec![player.clone()], random_players);
                new_games.push(random_game);
            }

            for game in new_games {
                let ranking = game.await;
                // update_results(game_results.clone(), ranking).await // todo should we update the result for these kind of test games?
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

async fn update_results(
    game_results: JsonLog<Vec<usize>>,
    ranking: Result<Vec<(PlayerId, usize)>, JoinError>,
) {
    match ranking {
        Ok(ranking) => {
            let mut result_map = game_results.data.lock().await;
            for (rank, (player, points)) in ranking.iter().enumerate() {
                result_map
                    .entry(player.name.clone())
                    .or_insert(vec![rank])
                    .push(rank);
            }
        }
        Err(_) => error!("og | game not properly finished"),
    }

    game_results.to_file().await;
}

pub fn spawn_game(
    ws_lobby: WebsocketLobby,
    ws_players: Vec<String>,
    random_players: Vec<String>,
) -> JoinHandle<Vec<(PlayerId, usize)>> {
    tokio::spawn(async move {
        let start_time = tokio::time::Instant::now();

        let mut all_ids: Vec<String> = Vec::new();
        all_ids.extend(ws_players.clone());
        all_ids.extend(random_players.clone());

        let mut ws_actions: Vec<WebsocketActions> = Vec::new();

        {
            let channel_for_ws_actions = ws_lobby.channels_for_ws_actions.lock().await;

            for id in &ws_players {
                let (sender, receiver) = channel_for_ws_actions.get(id).unwrap();
                let channels = (sender.clone(), Arc::clone(&receiver));

                ws_actions.push(WebsocketActions::new(id.clone(), channels));
            }
        }

        let mut random_actions: Vec<RandomPlayerActions> = Vec::new();
        for id in ws_players {
            random_actions.push(RandomPlayerActions::new(id.clone(), 25)); //todo change seed
        }

        let seed: u64 = 0; //todo change seed
        let game_handle = tokio::task::spawn_blocking(move || {
            println!("-------Default game--------\n");
            let mut all_actions: Vec<Box<dyn PlayerActions>> = Vec::new();
            all_actions.extend(
                ws_actions
                    .into_iter()
                    .map(|action: WebsocketActions| Box::new(action) as Box<dyn PlayerActions>),
            );
            all_actions.extend(
                random_actions
                    .into_iter()
                    .map(|action: RandomPlayerActions| Box::new(action) as Box<dyn PlayerActions>),
            );

            let mut game = Game::new_default_game(all_ids, all_actions, seed);
            game.num_players();
            println!("{}", game);

            game.num_players();

            let ranking = game.play().unwrap();

            println!("ranking: {:?}", ranking);
            tracing::event!(target: "ranking", Level::INFO, "{:?}", ranking);

            print!("game is done");
            ranking
        });

        let ranking = game_handle.await.unwrap();

        let mut timed_games = ws_lobby.time_last_n_games.lock().await;
        if timed_games.len() == ws_lobby.average_time_over_n_games {
            timed_games.remove(0);
        }
        let time_diff = tokio::time::Instant::now().duration_since(start_time);
        timed_games.push(time_diff);

        ranking
    })
}
