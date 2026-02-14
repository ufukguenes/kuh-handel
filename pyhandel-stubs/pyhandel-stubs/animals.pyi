from . import Value  # type: ignore

class Animal:
    value: Value
    def __init__(self, value: Value) -> None: ...

class AnimalSet:
    animal: Animal
    inflation: list[Value]

    def __init__(
        self,
        value: Value,
        inflation: list[Value],
    ) -> None: ...
