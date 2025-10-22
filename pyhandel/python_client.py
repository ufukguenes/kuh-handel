import threading
import asyncio
import pyhandel.pyhandel as pyh
import sys

bot_name = sys.argv[1]
if len(bot_name) < 2:
    print("provide bot name")
    quit()


print(bot_name)
bot = pyh.player.random_player.RandomPlayerActions(bot_name, 0)
client = pyh.client.Client(bot_name, "abcd", bot, "s://ufuk-guenes.com") # "://127.0.0.1:2000"



async def run(client):
    res = await client.register()
    await client.start()



try:
    asyncio.run(run(client))
except KeyboardInterrupt:
    print("Client shutdown")
except Exception as e:
    print("Error: ", e)