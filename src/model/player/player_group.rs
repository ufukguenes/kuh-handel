use crate::model::game_errors::GameError;
use crate::model::money::wallet::Wallet;
use crate::model::player::base_player::{Player, PlayerId};
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PlayerGroup {
    players: Vec<Rc<RefCell<Player>>>,
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
                    Rc::new(RefCell::new(Player::new(
                        id.clone(),
                        wallet.clone(),
                        player_action,
                    )))
                })
                .collect(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Rc<RefCell<Player>>> {
        self.players.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Rc<RefCell<Player>>> {
        self.players.iter_mut()
    }

    pub fn get(&self, index: usize) -> Result<Rc<RefCell<Player>>, GameError> {
        self.players
            .get(index)
            .ok_or(GameError::PlayerNotFound)
            .cloned()
    }

    pub fn get_by_id(&self, id: &PlayerId) -> Result<Rc<RefCell<Player>>, GameError> {
        for player in self.players.iter() {
            if player.borrow().id() == id {
                return Ok(Rc::clone(&player));
            }
        }
        Err(GameError::PlayerNotFound)
    }

    pub fn get_auction_players(&self, excluding: &PlayerId) -> Vec<Rc<RefCell<Player>>> {
        self.players
            .iter()
            .filter(|p| p.borrow().id() != excluding)
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.players.len()
    }
}
