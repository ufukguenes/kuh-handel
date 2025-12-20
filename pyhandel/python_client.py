import asyncio
import pyhandel as pyh
import sys
from abc import ABC, abstractmethod
from typing import override

bot_name = "ufuk"
bot_token = "ufuk"


class Bot(pyh.player.player_actions.PlayerActions):
    inner = pyh.player.simple_player.SimplePlayer(bot_name, 0.3)

    def _draw_or_trade(self):
        return self.inner._draw_or_trade()

    def _trade(self):
        return self.inner._trade()

    def _provide_bidding(self, state):
        return self.inner._provide_bidding(state)

    def _buy_or_sell(self, state):
        return self.inner._buy_or_sell(state)

    def _send_money_to_player(self, player, amount):
        return self.inner._send_money_to_player(player, amount)

    def _respond_to_trade(self, offer):
        return self.inner._respond_to_trade(offer)

    def _receive_game_update(self, update):
        return self.inner._receive_game_update(update)


bot = Bot()


client = pyh.client.Client(
    bot_name, bot_token, bot, "s://ufuk-guenes.com"
)  # "://127.0.0.1:2000"


async def run(client, num_rounds):
    for _ in range(num_rounds):
        await client.play_one_round()


try:
    asyncio.run(run(client, 10))
except KeyboardInterrupt:
    print("Client shutdown")
except Exception as e:
    print("Error: ", e)
