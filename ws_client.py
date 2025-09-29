import websocket
import time
# Connect to your server
ws = websocket.WebSocket()
ws.connect("ws://localhost:3000/game?player_id=ufuk")

time.sleep(3)
ws.connect("ws://localhost:3000/game?player_id=leon")

print("Connected to server")

# Send a test message
ws.send("Hello from Python client!")

# Receive a response
response = ws.recv()
print("Received:", response)

# Close the connection
ws.close()
