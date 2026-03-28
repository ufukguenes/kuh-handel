# kuh-handel-lib

Rust library for building bots for the [Kuh-Handel](https://ufuk-guenes.com/kuh-handel) online bot coding challenge.

## Usage

Implement the `PlayerActions` trait on your bot struct, then connect to the game server via `Client`:

```rust
use kuh_handel_lib::client::Client;
use kuh_handel_lib::messages::actions::{
    AuctionDecision, Bidding, InitialTrade, NoAction,
    PlayerTurnDecision, SendMoney, TradeOpponentDecision,
};
use kuh_handel_lib::messages::game_updates::{AuctionRound, GameUpdate, TradeOffer};
use kuh_handel_lib::player::base_player::PlayerId;
use kuh_handel_lib::player::player_actions::PlayerActions;
use kuh_handel_lib::Value;

#[derive(Default)]
struct Bot;

impl PlayerActions for Bot {
    fn _draw_or_trade(&mut self) -> PlayerTurnDecision { todo!() }
    fn _trade(&mut self) -> InitialTrade { todo!() }
    fn _provide_bidding(&mut self, state: AuctionRound) -> Bidding { todo!() }
    fn _buy_or_sell(&mut self, state: AuctionRound) -> AuctionDecision { todo!() }
    fn _send_money_to_player(&mut self, player: &PlayerId, amount: Value) -> SendMoney { todo!() }
    fn _respond_to_trade(&mut self, offer: TradeOffer) -> TradeOpponentDecision { todo!() }
    fn _receive_game_update(&mut self, update: GameUpdate) -> NoAction { todo!() }
}

#[tokio::main]
async fn main() {
    let bot = Box::new(Bot::default());
    let mut client = Client::new(
        "your_bot_name".to_string(),
        "your_token".to_string(),
        bot,
        "s://ufuk-guenes.com".to_string(),
        true,
    );
    client.play_one_round("pvp_games".to_string()).await;
}
```

## Documentation

- [API docs (docs.rs)](https://docs.rs/kuh-handel-lib)
- [Full tutorial](https://ufuk-guenes.com/kuh-handel/documentation)
- [Game rules](https://ufuk-guenes.com/kuh-handel/rules)

## License

MIT
