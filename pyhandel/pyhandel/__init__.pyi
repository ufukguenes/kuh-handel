from . import player as player
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
