from .player import player_actions as player_actions
from .player import simple_player as simple_player

from . import messages as messages
from . import animals as animals
from . import client as client

from typing import TypeAlias

Money: TypeAlias = int
Value: TypeAlias = int

__all__ = [
    "player",
    "messages",
    "animals",
    "client",
]
