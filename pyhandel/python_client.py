import threading
import asyncio
import pyhandel.pyhandel as pyh
import sys

bot_name = sys.argv[1]
if len(bot_name) < 2:
    print("provide bot name")
    quit()


print(bot_name)

bot = pyh.player.player_actions.PlayerActions(bot_name)
client = pyh.client.Client(bot_name, "abcd", bot, "://127.0.0.1:2000") # "://127.0.0.1:2000"


async def run(client, num_rounds):
    res = await client.register()
    for _ in range(num_rounds):
        await client.play_one_round()

try:
    asyncio.run(run(client, 10))
except KeyboardInterrupt:
    print("Client shutdown")
except Exception as e:
    print("Error: ", e)