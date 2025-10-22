import threading
import pyhandel
import asyncio
threads = []
clients = []

def run(client):
    async def main():
        await client.start()

    asyncio.run(main())


bot = pyhandel.player.random_player.RandomPlayerActions("ufuk", 0)
client = pyhandel.client.Client("ufuk", "abcd", bot)
res = asyncio.run(client.register())

clients.append(client)


bot = pyhandel.player.random_player.RandomPlayerActions("viola", 0)
client = pyhandel.client.Client("viola", "abcd", bot)
res = asyncio.run(client.register())
clients.append(client)

bot = pyhandel.player.random_player.RandomPlayerActions("johannes", 0)
client = pyhandel.client.Client("johannes", "abcd", bot)
res = asyncio.run(client.register())
clients.append(client)


bot = pyhandel.player.random_player.RandomPlayerActions("leon", 0)
client = pyhandel.client.Client("leon", "abcd", bot)
res = asyncio.run(client.register())
clients.append(client)



for client in clients:
    t = threading.Thread(target=run, args=(client,))
    threads.append(t)
    t.start()

for t in threads:
    t.join()