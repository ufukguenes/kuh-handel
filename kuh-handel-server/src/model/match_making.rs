use crate::model::game_logic::Game;
use crate::server_side_player::websocket_actions::WebsocketActions;

use kuh_handel_lib::player::player_actions::PlayerActions;
use kuh_handel_lib::player::random_player::RandomPlayerActions;

use axum::extract::ws::{self, Message};
pub use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tracing::{Level, info};

#[derive(Default, Clone)]
pub struct WebsocketLobby {
    pub channels_for_ws_actions:
        Arc<Mutex<BTreeMap<String, (Sender<Message>, Arc<Mutex<Receiver<Message>>>)>>>,
}

pub async fn organize_new_game(ws_lobby: WebsocketLobby) {
    // todo: better match making

    while ws_lobby.clone().channels_for_ws_actions.lock().await.len() < 4 {
        info!("og | waiting for more players to join");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    info!("og | enough players joined");
    loop {
        // todo how to handle if player drops connection? -> just use the backup action in the websocket actions?

        info!("og | creating new round of games");

        let new_ws_lobby = ws_lobby.clone();
        let first_game = spawn_game(
            new_ws_lobby.clone(),
            vec![String::from("ufuk"), String::from("leon")],
            vec![String::from("gregor")],
        );

        let new_ws_lobby = ws_lobby.clone();
        let second_game = spawn_game(
            new_ws_lobby,
            vec![String::from("johannes"), String::from("viola")],
            vec![String::from("fiete")],
        );

        let _wait = first_game.await;
        let _wait = second_game.await;

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

pub async fn spawn_game(
    ws_lobby: WebsocketLobby,
    ws_players: Vec<String>,
    random_players: Vec<String>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
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
            random_actions.push(RandomPlayerActions::new(id.clone(), 25)); //todo change see
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

            let results = game.play().unwrap();

            println!("ranking: {:?}", results);
            tracing::event!(target: "results", Level::INFO, "{:?}", results);

            print!("game is done");
        });

        let _ = game_handle.await;
    })
}
