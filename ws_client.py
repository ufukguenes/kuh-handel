import websocket
import time
# Connect to your server
ufuk_ws = websocket.WebSocket()
leon_ws = websocket.WebSocket()
ufuk_ws.connect("ws://localhost:3000/game?player_id=ufuk")

time.sleep(2)
leon_ws.connect("ws://localhost:3000/game?player_id=leon")

print("Connected to server")

# Send a test message
ufuk_ws.send("Hello from Python client!")

# Receive a response
response = ufuk_ws.recv()
print("Received:", response)

# Close the connection
ufuk_ws.close()
leon_ws.close()


