import pyhandel
import asyncio

bot = pyhandel.player.random_player.RandomPlayerActions("ufuk", 0)
client = pyhandel.client.Client("ufuk", "abcd", bot)

res = asyncio.run(client.register())
print(res)
print("hello")