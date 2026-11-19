"""The emulated platform: a bus, its devices and one hart, wired together."""

import os
from collections.abc import Callable
from dataclasses import dataclass
from typing import TYPE_CHECKING

from detrytus.asm import Image
from detrytus.bus import Bus
from detrytus.cpu import Hart
from detrytus.devices import CONSOLE_PAGE, POWER_PAGE, Console, Halt, PowerControl

if TYPE_CHECKING:
    from detrytus.rust import RustMachine


class StepLimit(Exception):
    pass


@dataclass
class Result:
    exit_code: int
    steps: int


class Machine:
    def __init__(
        self,
        ram_pages: int = 729,
        out: Callable[[str], None] | None = None,
        seed: int | None = None,
    ) -> None:
        self.bus = Bus(ram_pages)
        self.console = Console(out)
        self.bus.attach(CONSOLE_PAGE, self.console)
        self.bus.attach(POWER_PAGE, PowerControl())
        self.hart = Hart(self.bus, seed=seed)

    def load(self, image: Image) -> None:
        for addr, tryte in image.trytes.items():
            self.bus.poke(addr, tryte)

    def run(self, max_steps: int | None = None) -> Result:
        """Run until the program powers off; StepLimit if it runs too long."""
        steps = 0
        hart = self.hart
        try:
            while max_steps is None or steps < max_steps:
                hart.step()
                steps += 1
        except Halt as h:
            return Result(h.code, steps + 1)
        raise StepLimit(f"no power-off after {steps} steps")


BACKENDS = ("python", "rust")


def create(
    ram_pages: int = 729,
    out: Callable[[str], None] | None = None,
    seed: int | None = None,
    backend: str | None = None,
) -> "Machine | RustMachine":
    """A machine on the chosen core: "python" (the reference) or "rust"
    (the optional detrytus-rs package). With no backend given, the
    DETRYTUS_BACKEND environment variable chooses, else Python."""
    backend = backend or os.environ.get("DETRYTUS_BACKEND", "python")
    if backend == "python":
        return Machine(ram_pages, out, seed)
    if backend == "rust":
        from detrytus.rust import RustMachine

        return RustMachine(ram_pages, out, seed)
    raise ValueError(f"unknown backend {backend!r}: choose from {BACKENDS}")
