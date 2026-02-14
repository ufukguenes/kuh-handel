from . import messages as messages  # type: ignore
from . import animals as animals  # type: ignore
from . import client as client  # type: ignore
from . import player as player  # type: ignore

from typing import TypeAlias

Money: TypeAlias = int
Value: TypeAlias = int
Points: TypeAlias = int

__all__ = [
    "player",
    "messages",
    "animals",
    "client",
    "player",
]
