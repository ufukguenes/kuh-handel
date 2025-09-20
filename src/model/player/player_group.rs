use crate::model::game_errors::GameError;
use crate::model::money::wallet::Wallet;
use crate::model::player::base_player::Player;
use crate::model::player::player_actions::base_player_actions::PlayerActions;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PlayerGroup<T: PlayerActions> {
    players: Vec<Rc<RefCell<Player<T>>>>,
}

impl<T> PlayerGroup<T>
where
    T: PlayerActions,
{
    pub fn new(player_ids: Vec<String>, player_actions: Vec<T>, wallet: Wallet) -> Self {
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

    pub fn iter(&self) -> std::slice::Iter<'_, Rc<RefCell<Player<T>>>> {
        self.players.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Rc<RefCell<Player<T>>>> {
        self.players.iter_mut()
    }

    pub fn get(&self, index: usize) -> Result<Rc<RefCell<Player<T>>>, GameError> {
        self.players
            .get(index)
            .ok_or(GameError::PlayerNotFound)
            .cloned()
    }

    pub fn get_by_id_mut(&mut self, id: &PlayerId) -> Result<Rc<RefCell<Player<T>>>, GameError> {
        self.players
            .iter()
            .find(|p| p.borrow().id.name == id.name)
            .ok_or(GameError::PlayerNotFound)
            .cloned()
    }

    pub fn len(&self) -> usize {
        self.players.len()
    }
}
