pub mod messages {
    pub mod actions;
    pub mod game_updates;
    pub mod message_protocol;
}
pub mod animals;
pub mod client;

pub mod money {
    pub mod money;
    pub mod value;
    pub mod wallet;
}
pub mod player {
    pub mod base_player;
    pub mod player_actions;
    pub mod player_error;
    pub mod random_player;
}

#[cfg(test)]
mod tests {}
