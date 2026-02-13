from enum import Enum
from ..player import PlayerId
from ..animals import Animal
from .. import Money, Value
from .actions import Bidding


class AuctionRound:
    host: PlayerId
    animal: Animal
    bids: list[tuple[PlayerId, Bidding]]

class TradeOffer: 
    challenger: PlayerId
    animal: Animal
    animal_count: int
    challenger_card_offer: int

class GameUpdate: ...

class AuctionKind: ...
class MoneyTransfer: ...
class MoneyTrade: ...