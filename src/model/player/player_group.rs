use crate::model::game_errors::GameError;
use crate::model::money::wallet::Wallet;
use crate::model::player::base_player::{Player, PlayerId};
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct PlayerGroup {
    players: Vec<Arc<Mutex<Player>>>,
}

impl PlayerGroup {
    pub fn new(
        player_ids: Vec<String>,
        player_actions: Vec<Box<dyn PlayerActions>>,
        wallet: Wallet,
    ) -> Self {
        PlayerGroup {
            players: player_ids
                .iter()
                .zip(player_actions)
                .map(|(id, player_action)| {
                    Arc::new(Mutex::new(Player::new(
                        id.clone(),
                        wallet.clone(),
                        player_action,
                    )))
                })
                .collect(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Arc<Mutex<Player>>> {
        self.players.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Arc<Mutex<Player>>> {
        self.players.iter_mut()
    }

    pub fn get(&self, index: usize) -> Result<Arc<Mutex<Player>>, GameError> {
        self.players
            .get(index)
            .ok_or(GameError::PlayerNotFound)
            .cloned()
    }

    pub async fn get_by_id_mut(&mut self, id: &PlayerId) -> Result<Arc<Mutex<Player>>, GameError> {
        for player in self.players.iter() {
            if player.lock().await.id() == id.name() {
                return Ok(Arc::clone(&player));
            }
        }
        Err(GameError::PlayerNotFound)
    }

    pub fn len(&self) -> usize {
        self.players.len()
    }
}
