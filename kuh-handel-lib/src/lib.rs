/// modules defining different messages
pub mod messages {
    pub mod actions;
    pub mod game_updates;
    pub mod message_protocol;
}

/// modules used for managing animals
pub mod animals;

/// used to connect to a game server and handles messaging
pub mod client;

/// modules used for implementing a bot
pub mod player {
    pub mod base_player;
    pub mod player_actions;
    pub mod player_error;
    pub mod random_player;
    pub mod simple_player;
    pub mod wallet;
}

/// represents a valid bill
pub type Money = usize;

/// represents the value of something, e.g. an amount that needs to be payed or the value of an animal
pub type Value = usize;

#[cfg(test)]
mod tests {}
