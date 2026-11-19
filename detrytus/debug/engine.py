"""The debugger's engine: run control, breakpoints, watchpoints, inspection.

It drives a Machine one step at a time and decides when to stop. It has
no user interface; shell.py is one, and tests drive it directly.
"""

import re
from collections.abc import Callable
from dataclasses import dataclass, field

from detrytus import isa
from detrytus.asm import Image
from detrytus.devices import Halt
from detrytus.disasm import disassemble
from detrytus.expr import evaluate
from detrytus.machine import Machine
from detrytus.trap import UnhandledTrap

DEFAULT_LIMIT = 10_000_000
_REGISTER = re.compile(r"\$(r-?\d+|[a-z][a-z0-9]*)")


@dataclass
class Stop:
    """Why execution stopped, for the user to read."""

    reason: str  # 'step', 'breakpoint', 'watchpoint', 'trap', 'halted', ...
    message: str = ""
    exit_code: int | None = None


@dataclass
class Watch:
    first: int  # lowest tryte watched
    last: int  # highest tryte watched
    kinds: str  # any of 'r' and 'w'
    value: int | None  # last value seen, for old/new reports

    def hit(self, addr: int, width: int, kind: str) -> bool:
        lo, hi = (addr - 1, addr + 1) if width == 3 else (addr, addr)
        return (
            lo <= self.last and hi >= self.first and any(k in self.kinds for k in kind)
        )


@dataclass
class _Pending:
    """What happened during one step, gathered by the hart's hooks."""

    accesses: list[tuple[int, int, str]] = field(default_factory=list)
    traps: list[tuple[int, int, int]] = field(default_factory=list)


class Debugger:
    def __init__(
        self,
        image: Image,
        ram_pages: int = 729,
        out: Callable[[str], None] | None = None,
    ) -> None:
        self.image = image
        self.ram_pages = ram_pages
        self.out = out
        self.breakpoints: dict[int, int] = {}  # id -> address
        self.watches: dict[int, Watch] = {}
        self.catch: set[int] | None = None  # trap causes to stop on; empty = all
        self._ids = 0
        self.restart()

    # --- the machine -------------------------------------------------------

    def restart(self) -> None:
        """A fresh machine with the image loaded. Breakpoints stay."""
        self.machine = Machine(ram_pages=self.ram_pages, out=self.out, seed=1)
        self.machine.load(self.image)
        self.hart = self.machine.hart
        self._pending = _Pending()
        self.hart.on_access = lambda a, w, k: self._pending.accesses.append((a, w, k))
        self.hart.on_trap = lambda c, v, p: self._pending.traps.append((c, v, p))
        self.halted: int | None = None
        for watch in self.watches.values():
            watch.value = self._watched_value(watch)

    # --- run control -------------------------------------------------------

    def step(self, count: int = 1) -> Stop:
        """Run count instructions, or fewer if something stops it."""
        for _ in range(count):
            stop = self._step_once()
            if stop is not None:
                return stop
        return Stop("step")

    def cont(self, limit: int = DEFAULT_LIMIT) -> Stop:
        """Run until a breakpoint, watchpoint, caught trap or power-off.

        The first instruction always runs, so continuing from a
        breakpoint does not stop on it again.
        """
        for n in range(limit):
            if n and self.hart.pc in self.breakpoints.values():
                return Stop("breakpoint", self._breakpoint_message())
            stop = self._step_once()
            if stop is not None:
                return stop
        return Stop("limit", f"still running after {limit} steps")

    def next(self, limit: int = DEFAULT_LIMIT) -> Stop:
        """Step, treating a call as one step: run until it returns."""
        d = self._decoded(self.hart.pc)
        is_call = d is not None and d.op.name in ("jal", "jalr") and d.rd != 0
        if not is_call:
            return self.step()
        return_to = self.hart.pc + 3
        self.breakpoints[0] = return_to  # id 0: a temporary breakpoint
        try:
            stop = self.cont(limit)
        finally:
            del self.breakpoints[0]
        if stop.reason == "breakpoint" and self.hart.pc == return_to:
            return Stop("step")
        return stop

    def _step_once(self) -> Stop | None:
        if self.halted is not None:
            return Stop("halted", f"the machine has powered off (exit {self.halted})")
        self._pending = _Pending()
        try:
            self.hart.step()
        except Halt as h:
            self.halted = h.code
            return Stop("halted", f"powered off with exit code {h.code}", h.code)
        except UnhandledTrap as e:
            self.halted = -1
            return Stop("unhandled", str(e))
        except NotImplementedError as e:
            return Stop("unimplemented", f"not implemented yet: {e}")
        return self._caught_trap() or self._watch_hit()

    def _caught_trap(self) -> Stop | None:
        for cause, tval, tpc in self._pending.traps:
            if self.catch is not None and (not self.catch or cause in self.catch):
                return Stop(
                    "trap",
                    f"trap cause {cause:+d} at {self.where(tpc)}, tval {tval}",
                )
        return None

    def _watch_hit(self) -> Stop | None:
        for addr, width, kind in self._pending.accesses:
            for wid, watch in self.watches.items():
                if watch.hit(addr, width, kind):
                    old, watch.value = watch.value, self._watched_value(watch)
                    verb = {"r": "read", "w": "write", "rw": "update"}[kind]
                    change = "" if kind == "r" else f": {old} -> {watch.value}"
                    return Stop(
                        "watchpoint",
                        f"watchpoint {wid}: {verb} at {self.where(addr)}{change}",
                    )
        return None

    def _breakpoint_message(self) -> str:
        pc = self.hart.pc
        ids = [i for i, a in self.breakpoints.items() if a == pc and i]
        label = f"breakpoint {ids[0]}" if ids else "returned"
        return f"{label} at {self.where(pc)}"

    # --- breakpoints, watchpoints, catching ----------------------------------

    def _new_id(self) -> int:
        self._ids += 1
        return self._ids

    def add_breakpoint(self, expr: str) -> tuple[int, int]:
        addr = self.evaluate(expr)
        bid = self._new_id()
        self.breakpoints[bid] = addr
        return bid, addr

    def add_watch(self, expr: str, kinds: str = "w") -> tuple[int, Watch]:
        """Watch the word centred on an aligned address, else one tryte."""
        if not kinds or set(kinds) - {"r", "w"}:
            raise ValueError("watch kinds are r, w or rw")
        addr = self.evaluate(expr)
        span = 1 if addr % 3 == 0 else 0
        watch = Watch(addr - span, addr + span, kinds, None)
        watch.value = self._watched_value(watch)
        wid = self._new_id()
        self.watches[wid] = watch
        return wid, watch

    def delete(self, ident: int | None = None) -> None:
        if ident is None:
            self.breakpoints = {k: v for k, v in self.breakpoints.items() if k == 0}
            self.watches.clear()
            return
        if (
            self.breakpoints.pop(ident, None) is None
            and self.watches.pop(ident, None) is None
        ):
            raise ValueError(f"no breakpoint or watchpoint {ident}")

    # --- inspection ----------------------------------------------------------

    def evaluate(self, text: str) -> int:
        """An assembler expression, where $name is a register and $pc the pc."""

        def register(m: re.Match[str]) -> str:
            name = m.group(1)
            if name == "pc":
                return str(self.hart.pc)
            if name not in isa.REG_NAMES:
                raise ValueError(f"no register ${name}")
            return f"({self.hart.reg(name)})"

        return evaluate(_REGISTER.sub(register, text), self.image.symbols, self.hart.pc)

    def where(self, addr: int) -> str:
        """addr, with the nearest label at or below it: '42 <loop+3>'."""
        best = None
        for name, value in self.image.labels.items():
            if value <= addr and (best is None or value > best[1]):
                best = (name, value)
        if best is None:
            return str(addr)
        offset = addr - best[1]
        return f"{addr} <{best[0]}{f'+{offset}' if offset else ''}>"

    def peek(self, addr: int, width: int) -> int | None:
        """Memory as the current ring sees it, with no side effects."""
        physical = self.hart.probe(addr)
        return None if physical is None else self.machine.bus.peek(physical, width)

    def _watched_value(self, watch: Watch) -> int | None:
        if watch.last > watch.first:
            return self.peek(watch.first + 1, 3)
        return self.peek(watch.first, 1)

    def _decoded(self, addr: int) -> isa.Decoded | None:
        word = self.peek(addr, 3)
        if word is None:
            return None
        try:
            return isa.decode(word)
        except isa.IllegalInstruction:
            return None

    def disassemble(self, addr: int, count: int) -> list[tuple[int, str]]:
        lines = []
        for i in range(count):
            a = addr + 3 * i
            word = self.peek(a, 3)
            lines.append((a, "<unmapped>" if word is None else disassemble(word, a)))
        return lines
