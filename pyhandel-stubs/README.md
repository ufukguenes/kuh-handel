# pyhandel-stubs

Type stubs for [pyhandel](https://pypi.org/project/pyhandel/), the Python client library for the [Kuh-Handel](https://ufuk-guenes.com/kuh-handel) bot coding challenge.

Installing these stubs enables type checking and autocompletion in your IDE for pyhandel objects, which is especially useful for navigating the game state and message types.

## Installation

```bash
pip install pyhandel pyhandel-stubs
```

We strongly recommend using a type checker like [mypy](https://mypy.readthedocs.io/) alongside the stubs to get the most out of them.

## Note

Due to the way the Rust bindings are generated, some enum variants may appear to be infinitely chainable in type hints (e.g. `AuctionKind.NormalAuction.NormalAuction...`). This is a known quirk, only one level of access is valid at runtime.

## License

MIT
