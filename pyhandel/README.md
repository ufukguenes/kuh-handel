# pyhandel

Python wrapper for building bots for the [Kuh-Handel](https://ufuk-guenes.com/kuh-handel) online bot coding challenge. Wraps the core Rust implementation via PyO3.

## Installation

```bash
pip install pyhandel
```

Type stubs for better IDE support:

```bash
pip install pyhandel pyhandel-stubs
```

## Usage

Implement the `PlayerActions` class and connect to the game server via `Client`:

```python
import asyncio
from pyhandel.client import Client
from pyhandel.messages.actions import (
    AuctionDecision, Bidding, InitialTrade, NoAction,
    PlayerTurnDecision, SendMoney, TradeOpponentDecision,
)
from pyhandel.messages.game_updates import AuctionRound, GameUpdate, TradeOffer
from pyhandel.player.player_actions import PlayerActions


class Bot(PlayerActions):

    def setup(self):
        pass  # initialize your bot here

    def _draw_or_trade(self) -> PlayerTurnDecision:
        raise NotImplementedError

    def _trade(self) -> InitialTrade:
        raise NotImplementedError

    def _provide_bidding(self, state: AuctionRound) -> Bidding:
        raise NotImplementedError

    def _buy_or_sell(self, state: AuctionRound) -> AuctionDecision:
        raise NotImplementedError

    def _send_money_to_player(self, player: str, amount: int) -> SendMoney:
        raise NotImplementedError

    def _respond_to_trade(self, offer: TradeOffer) -> TradeOpponentDecision:
        raise NotImplementedError

    def _receive_game_update(self, update: GameUpdate) -> NoAction:
        raise NotImplementedError


async def run(client, num_rounds):
    for _ in range(num_rounds):
        await client.play_one_round("pvp_games")

if __name__ == "__main__":
    bot = Bot()
    bot.setup()
    client = Client("your_bot_name", "your_token", bot, "s://ufuk-guenes.com", True)
    asyncio.run(run(client, 1))
```

## Documentation

- [Full tutorial](https://ufuk-guenes.com/kuh-handel/documentation)
- [Game rules](https://ufuk-guenes.com/kuh-handel/rules)
- [API reference (Rust)](https://docs.rs/kuh-handel-lib)

## License

MIT
