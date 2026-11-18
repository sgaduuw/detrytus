"""The emulator's platform devices. Layout is in docs/platform.md."""

from collections import deque
from collections.abc import Callable

CONSOLE_PAGE = -1
POWER_PAGE = -2

PAGE = 3**9
CONSOLE_OUT = CONSOLE_PAGE * PAGE
CONSOLE_IN = CONSOLE_OUT + 3
POWER_OFF = POWER_PAGE * PAGE


class Halt(Exception):
    """The program asked the machine to power off."""

    def __init__(self, code: int) -> None:
        super().__init__(f"halt with code {code}")
        self.code = code


class Console:
    """A character console: one code point per write, one per read.

    Writing a Unicode code point to CONSOLE_OUT prints it. Reading
    CONSOLE_IN returns the next queued input code point, or -1 when there
    is none, so a program can poll without blocking the emulator.
    """

    def __init__(self, out: Callable[[str], None] | None = None) -> None:
        self.output: list[str] = []
        self._out = out
        self.input: deque[int] = deque()

    def feed(self, text: str) -> None:
        self.input.extend(ord(c) for c in text)

    def read(self, addr: int, width: int) -> int:
        if addr == CONSOLE_IN:
            return self.input.popleft() if self.input else -1
        return 0

    def write(self, addr: int, width: int, value: int) -> None:
        if addr == CONSOLE_OUT and 0 <= value <= 0x10FFFF:
            ch = chr(value)
            self.output.append(ch)
            if self._out is not None:
                self._out(ch)

    @property
    def text(self) -> str:
        return "".join(self.output)


class PowerControl:
    """Writing any value to POWER_OFF stops the machine with that exit code."""

    def read(self, addr: int, width: int) -> int:
        return 0

    def write(self, addr: int, width: int, value: int) -> None:
        if addr == POWER_OFF:
            raise Halt(value)
