pub mod messages {
    pub mod actions;
    pub mod game_updates;
    pub mod message_protocol;
}
pub mod animals;
pub mod client;

pub mod player {
    pub mod base_player;
    pub mod player_actions;
    pub mod player_error;
    pub mod random_player;
    pub mod wallet;
}

pub type Money = usize;
pub type Value = usize;

#[cfg(test)]
mod tests {}
