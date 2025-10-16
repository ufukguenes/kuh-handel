#[derive(Debug)]
pub enum GameError {
    PlayerNotFound,
    MoneyNotAvailable,
    AnimalsNotAvailable,
    InvalidAction,
    InvalidState,

    InvalidMoneyAtEnd,
    InvalidAnimalsAtEnd,
}
