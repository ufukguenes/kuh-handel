from ..player import PlayerId  # type: ignore
from ..animals import Animal  # type: ignore
from .. import Money, Value  # type: ignore

class NoAction:
    class Ok(NoAction): ...

class InitialTrade:
    opponent: PlayerId
    animal: Animal
    animal_count: int
    amount: list[Money]

    def __init__(
        self, opponent: PlayerId, animal: Animal, animal_count: int, amount: list[Money]
    ) -> None: ...

class PlayerTurnDecision:
    class Draw(PlayerTurnDecision): ...

    class Trade(PlayerTurnDecision):
        initial_trade: InitialTrade
        def __init__(self, initial_trade: InitialTrade) -> None: ...
        __match_args__ = ("initial_trade",)

class AuctionDecision:
    class Buy(AuctionDecision): ...
    class Sell(AuctionDecision): ...

class TradeOpponentDecision:
    class Accept(TradeOpponentDecision): ...

    class CounterOffer(TradeOpponentDecision):
        money_list: list[Money]
        def __init__(self, money_list: list[Money]) -> None: ...
        __match_args__ = ("money_list",)

class SendMoney:
    class WasBluff:
        def __init__(self) -> None: ...

    class Amount:
        money_list: list[Money]
        def __init__(self, money_list: list[Money]) -> None: ...
        __match_args__ = ("money_list",)

class Bidding:
    class Pass:
        def __init__(self) -> None: ...

    class Bid:
        value: Value
        def __init__(self, value: Value) -> None: ...
