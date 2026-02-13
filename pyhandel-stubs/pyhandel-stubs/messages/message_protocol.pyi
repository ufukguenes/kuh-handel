from typing import Union
from .actions import (
    AuctionDecision,
    Bidding,
    InitialTrade,
    NoAction,
    PlayerTurnDecision,
    SendMoney,
    TradeOpponentDecision,
)
from .game_updates import AuctionRound, GameUpdate, TradeOffer
from ..player import PlayerId
from .. import Value

class ActionMessage:
    class PlayerTurnDecision(ActionMessage):
        decision: PlayerTurnDecision
        def __init__(self, decision: PlayerTurnDecision) -> None: ...
        __match_args__ = ("decision",)

    class InitialTrade(ActionMessage):
        decision: InitialTrade
        def __init__(self, decision: InitialTrade) -> None: ...
        __match_args__ = ("decision",)

    class Bidding(ActionMessage):
        decision: Bidding
        def __init__(self, decision: Bidding) -> None: ...
        __match_args__ = ("decision",)

    class AuctionDecision(ActionMessage):
        decision: AuctionDecision
        def __init__(self, decision: AuctionDecision) -> None: ...
        __match_args__ = ("decision",)

    class SendMoney(ActionMessage):
        decision: SendMoney
        def __init__(self, decision: SendMoney) -> None: ...
        __match_args__ = ("decision",)

    class TradeOpponentDecision(ActionMessage):
        decision: TradeOpponentDecision
        def __init__(self, decision: TradeOpponentDecision) -> None: ...
        __match_args__ = ("decision",)

    class NoAction(ActionMessage):
        decision: NoAction
        def __init__(self, decision: NoAction) -> None: ...
        __match_args__ = ("decision",)

class StateMessage:
    class DrawOrTrade(StateMessage):
        def __init__(self) -> None: ...

    class Trade(StateMessage):
        def __init__(self) -> None: ...

    class ProvideBidding(StateMessage):
        state: AuctionRound
        def __init__(self, state: AuctionRound) -> None: ...
        __match_args__ = ("state",)

    class BuyOrSell(StateMessage):
        state: AuctionRound
        def __init__(self, state: AuctionRound) -> None: ...
        __match_args__ = ("state",)

    class SendMoney(StateMessage):
        player_id: PlayerId
        amount: Value
        def __init__(self, player_id: PlayerId, amount: Value) -> None: ...
        __match_args__ = ("player_id", "amount")

    class RespondToTrade(StateMessage):
        offer: TradeOffer
        def __init__(self, offer: TradeOffer) -> None: ...
        __match_args__ = ("offer",)

    class GameUpdate(StateMessage):
        update: GameUpdate
        def __init__(self, update: GameUpdate) -> None: ...
        __match_args__ = ("update",)
