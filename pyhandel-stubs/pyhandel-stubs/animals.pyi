from . import Value

class Animal:
    value: Value
    def __init__(self, value: Value) -> None: ...

class AnimalSet:
    animal: Animal
    inflation: list[Value]
    draw_count: int
    animals: list[Animal]

    def __init__(
        self,
        value: Value,
        inflation: list[Value],
    ) -> None: ...
