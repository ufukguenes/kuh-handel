from .player.player_actions import PlayerActions  # type: ignore

class Client:
    def __init__(
        self,
        name: str,
        token: str,
        bot: PlayerActions,
        base_url: str,
        raise_faulty_action_warning: bool,
    ): ...
    async def register(self): ...
    async def play_one_round(self, game_type_url: str): ...
