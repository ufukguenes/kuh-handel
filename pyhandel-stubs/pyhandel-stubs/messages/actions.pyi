from enum import Enum
from ..player import PlayerId
from ..animals import Animal
from .. import Money, Value
from dataclasses import dataclass
from typing import Union



class NoAction:
    class Ok(NoAction): ...


class InitialTrade:
    opponent: PlayerId
    animal: Animal
    animal_count: int
    amount: list[Money]

class PlayerTurnDecision:
    
    @staticmethod
    def Draw() -> PlayerTurnDecision: ...

    @staticmethod
    def Trade(initial_trade: InitialTrade) -> PlayerTurnDecision: ... 


class Buy:
    def __init__(self) -> None: ...

class Sell:
    def __init__(self) -> None: ...

class AuctionDecision:
    class Buy(AuctionDecision): ...
    class Sell(AuctionDecision): ...



class TradeOpponentDecision:
    class Accept(TradeOpponentDecision): ...
    class CounterOffer(TradeOpponentDecision):
        data: list[Money]
        def __init__(self, data: list[Money]) -> None: ...
        __match_args__ = ("data")


class WasBluff:
    def __init__(self) -> None: ...

class Amount:
    data: list[Money]
    def __init__(self, data: list[Money]) -> None: ...

SendMoney = Union[WasBluff, Amount]


class Pass:
    def __init__(self) -> None: ...

class Bid:
    data: Value
    def __init__(self, data: Value) -> None: ...

Bidding = Union[Pass, Bid]
