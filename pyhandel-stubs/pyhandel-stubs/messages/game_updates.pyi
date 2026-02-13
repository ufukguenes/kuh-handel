from enum import Enum
from ..player import PlayerId
from ..player.wallet import Wallet
from ..animals import Animal, AnimalSet
from .. import Money, Value, Points
from .actions import Bidding

class AuctionRound:
    host: PlayerId
    animal: Animal
    bids: list[tuple[PlayerId, Bidding]]

    def __init__(
        self,
        host: PlayerId,
        animal: Animal,
        bids: list[tuple[PlayerId, Bidding]],
    ) -> None: ...

class TradeOffer:
    challenger: PlayerId
    animal: Animal
    animal_count: int
    challenger_card_offer: int

    def __init__(
        self,
        challenger: PlayerId,
        animal: Animal,
        animal_count: int,
        challenger_card_offer: int,
    ) -> None: ...

class GameUpdate:
    class Auction(GameUpdate):
        auction_kind: AuctionKind
        def __init__(self, auction_kind: AuctionKind) -> None: ...
        __match_args__ = ("auction_kind",)

    class Trade(GameUpdate):
        challenger: PlayerId
        opponent: PlayerId
        animal: Animal
        animal_count: int
        receiver: PlayerId
        money_trade: MoneyTrade

        def __init__(
            self,
            challenger: PlayerId,
            opponent: PlayerId,
            animal: Animal,
            animal_count: int,
            receiver: PlayerId,
            money_trade: MoneyTrade,
        ) -> None: ...

        __match_args__ = (
            "challenger",
            "opponent",
            "animal",
            "animal_count",
            "receiver",
            "money_trade",
        )

    class Start(GameUpdate):
        wallet: Wallet
        players_in_turn_order: list[PlayerId]
        animals: list[AnimalSet]

        def __init__(
            self,
            wallet: Wallet,
            players_in_turn_order: list[PlayerId],
            animals: list[AnimalSet],
        ) -> None: ...

        __match_args__ = (
            "wallet",
            "players_in_turn_order",
            "animals",
        )

    class End(GameUpdate):
        ranking: list[tuple[PlayerId, Points]]
        def __init__(self, ranking: list[tuple[PlayerId, Points]]) -> None: ...

        __match_args__ = ("ranking",)

    class ExposePlayer(GameUpdate):
        player: PlayerId
        wallet: Wallet

        def __init__(self, player: PlayerId, wallet: Wallet) -> None: ...

        __match_args__ = (
            "player",
            "wallet",
        )

    class Inflation(GameUpdate):
        money: Money
        def __init__(self, money: Money) -> None: ...

        __match_args__ = ("money",)

class AuctionKind:
    class NoBiddings(AuctionKind):
        host_id: PlayerId
        animal: Animal
        def __init__(self, host_id: PlayerId, animal: Animal) -> None: ...

        __match_args__ = ("host_id", "animal")

    class NormalAuction(AuctionKind):
        rounds: AuctionRound
        from_player: PlayerId
        to_player: PlayerId
        money_transfer: MoneyTransfer

        def __init__(
            self,
            rounds: AuctionRound,
            from_player: PlayerId,
            to_player: PlayerId,
            money_transfer: MoneyTransfer,
        ) -> None: ...

        __match_args__ = ("rounds", "from_player", "to_player", "money_transfer")

class MoneyTransfer:
    class Public(MoneyTransfer):
        card_amount: int
        min_value: Value

        def __init__(
            self,
            card_amount: int,
            min_value: Value,
        ) -> None: ...

        __match_args__ = ("card_amount", "min_value")

class MoneyTrade:
    class Public(MoneyTrade):
        challenger_card_offer: int
        opponent_card_offer: tuple[int | None]

        def __init__(
            self,
            challenger_card_offer: int,
            opponent_card_offer: tuple[int | None],
        ) -> None: ...

        __match_args__ = ("card_amount", "min_value")

    class Private(MoneyTrade):
        challenger_card_offer: list[Money]
        opponent_card_offer: tuple[list[Money] | None]

        def __init__(
            self,
            challenger_card_offer: list[Money],
            opponent_card_offer: tuple[list[Money] | None],
        ) -> None: ...

        __match_args__ = ("challenger_card_offer", "opponent_card_offer")
