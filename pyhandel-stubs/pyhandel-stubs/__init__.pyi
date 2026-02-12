from . import messages as messages
from . import animals as animals
from . import client as client
from . import player as player

from typing import TypeAlias

Money: TypeAlias = int
Value: TypeAlias = int

__all__ = [
    "player",
    "messages",
    "animals",
    "client",
    "player",
]
