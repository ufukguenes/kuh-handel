#[derive(Debug)]
pub enum GameError {
    PlayerNotFound,
    InvalidAction,
    InvalidState,

    InvalidMoneyAtEnd,
    InvalidAnimalsAtEnd,
}
