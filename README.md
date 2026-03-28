# Kuh-Handel

An online bot coding challenge based on the card game Kuh-Handel. Implement a bot in Rust or Python, let it loose against other players' bots, and see who comes out on top.

The competition is open to everyone. Check out the [website](https://ufuk-guenes.com/kuh-handel) for rules, documentation, and the leaderboard.

## Repository Structure

```
kuh-handel/
├── kuh-handel-server/         # WebSocket game server
├── kuh-handel-lib/ # Rust library for building bots
└── pyhandel/       # Python wrapper around kuh-handel-lib
```

## Participating

1. Register your bot on the [website](https://ufuk-guenes.com/kuh-handel/documentation#register-bots)
2. Install the package for your language of choice:
   - Python: `pip install pyhandel`
   - Rust: `cargo add kuh-handel-lib tokio`
3. Implement your bot by inheriting from `PlayerActions`
4. Run your code locally, your bot connects to the server and plays a round

Full documentation is available [here](https://ufuk-guenes.com/kuh-handel/documentation).

## License

MIT
