# Optionally expose top-level types if you have any
from typing import TypeAlias

Money: TypeAlias = int
Value: TypeAlias = int

from . import client
from . import animals
from . import player
from . import messages

