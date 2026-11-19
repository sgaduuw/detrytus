"""The Rust core behind the Python core's interface.

The Rust core is the optional `detrytus-rs` package (rust/py), built by
maturin; install it with `uv pip install ./rust/py`. This module wraps
it in the parts of the Python Machine that code outside the emulator
uses: load, run, the console, registers and CSRs by name, and physical
memory. The debugger and tracing need the Python core's hooks and stay
on it. See docs/rust-plan.md, phase 4.
"""

import os
from collections.abc import Callable
from typing import Any

from detrytus.asm import Image
from detrytus.bus import BusFault
from detrytus.machine import Result, StepLimit
from detrytus.trap import UnhandledTrap

try:
    import detrytus_rs
except ImportError:  # the optional package is not installed
    detrytus_rs = None

# Console output is passed on between chunks of this many steps.
CHUNK = 100_000


def available() -> bool:
    return detrytus_rs is not None


class RustHart:
    def __init__(self, core: Any) -> None:
        self._core = core

    def x(self, r: int) -> int:
        value: int = self._core.x(r)
        return value

    def reg(self, name: str) -> int:
        from detrytus import isa

        return self.x(isa.REG_NAMES[name])

    def csr(self, name: str) -> int:
        """A CSR by name, read with ring +1's privilege."""
        value: int | None = self._core.csr(name)
        if value is None:
            raise KeyError(name)
        return value

    @property
    def pc(self) -> int:
        value: int = self._core.pc
        return value

    @property
    def ring(self) -> int:
        value: int = self._core.ring
        return value

    @property
    def instret(self) -> int:
        value: int = self._core.instret
        return value


class RustBus:
    def __init__(self, core: Any) -> None:
        self._core = core

    def read(self, addr: int, width: int, kind: str = "load") -> int:
        value: int | None = self._core.read(addr, width, kind)
        if value is None:
            raise BusFault(addr, kind)
        return value


class RustConsole:
    def __init__(self, core: Any) -> None:
        self._core = core

    @property
    def text(self) -> str:
        value: str = self._core.console_text()
        return value

    def feed(self, text: str) -> None:
        self._core.console_feed(text)


class RustMachine:
    """A Machine on the Rust core: the same constructor, load and run."""

    def __init__(
        self,
        ram_pages: int = 729,
        out: Callable[[str], None] | None = None,
        seed: int | None = None,
    ) -> None:
        if detrytus_rs is None:
            raise RuntimeError(
                "the Rust backend needs the detrytus-rs package: "
                "uv pip install ./rust/py"
            )
        if seed is None:
            seed = int.from_bytes(os.urandom(4))
        self.core = detrytus_rs.Machine(ram_pages, seed)
        self.hart = RustHart(self.core)
        self.bus = RustBus(self.core)
        self.console = RustConsole(self.core)
        self._out = out
        self._printed = 0

    def load(self, image: Image) -> None:
        self.core.load(image.trytes)

    def _flush(self) -> None:
        if self._out is not None:
            text = self.console.text
            for ch in text[self._printed :]:
                self._out(ch)
            self._printed = len(text)

    def run(self, max_steps: int | None = None) -> Result:
        """Run until the program powers off, as Machine.run does."""
        done = 0
        while max_steps is None or done < max_steps:
            chunk = CHUNK if max_steps is None else min(CHUNK, max_steps - done)
            kind, numbers, message = self.core.run(chunk)
            self._flush()
            if kind == "exit":
                code, steps = numbers
                return Result(code, done + steps)
            if kind == "unhandled":
                raise UnhandledTrap(*numbers)
            if kind == "unsupported":
                raise NotImplementedError(message)
            done += chunk
        raise StepLimit(f"no power-off after {done} steps")
