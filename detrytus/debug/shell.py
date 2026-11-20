"""The interactive debugger: a command shell over the engine.

Built on the standard library's cmd module, which brings the prompt,
line editing where readline exists, and help from docstrings.
"""

import cmd
from typing import IO

from detrytus import isa
from detrytus.debug.engine import Debugger, Stop
from detrytus.expr import ExprError
from detrytus.trap import Trap
from detrytus.trits import format_trits

ALIASES = {
    "s": "step",
    "n": "next",
    "c": "continue",
    "b": "break",
    "d": "delete",
    "w": "watch",
    "p": "print",
    "x": "examine",
    "q": "quit",
    "i": "info",
}

CAUSES = {
    "illegal": isa.ILLEGAL,
    "misaligned": isa.MISALIGNED,
    "access": isa.ACCESS_FAULT,
    "syscall": isa.SYSCALL,
    "breakpoint": isa.BREAKPOINT,
    "fetch-fault": isa.INSTR_PAGE_FAULT,
    "load-fault": isa.LOAD_PAGE_FAULT,
    "store-fault": isa.STORE_PAGE_FAULT,
    "timer": isa.IRQ_TIMER,
}


class UsageError(Exception):
    pass


class DebugShell(cmd.Cmd):
    intro = "detrytus debugger. 'help' lists commands; 'help CMD' explains one."
    prompt = "(dt) "

    def __init__(
        self,
        debugger: Debugger,
        stdin: IO[str] | None = None,
        stdout: IO[str] | None = None,
    ) -> None:
        super().__init__(stdin=stdin, stdout=stdout)
        if stdin is not None:
            self.use_rawinput = False
        self.dbg = debugger

    # --- plumbing ------------------------------------------------------------

    def say(self, text: str = "") -> None:
        self.stdout.write(text + "\n")

    def precmd(self, line: str) -> str:
        word, _, rest = line.strip().partition(" ")
        return f"{ALIASES.get(word, word)} {rest}".strip()

    def onecmd(self, line: str) -> bool:
        try:
            return super().onecmd(line)
        except (UsageError, ExprError, ValueError) as e:
            self.say(f"error: {e}")
            return False

    def emptyline(self) -> bool:
        return False  # do not repeat the last command: stepping by accident hurts

    def default(self, line: str) -> None:
        self.say(f"unknown command {line.split()[0]!r}; 'help' lists them")

    def report(self, stop: Stop) -> None:
        if stop.message:
            self.say(
                f"stopped: {stop.message}" if stop.reason != "step" else stop.message
            )
        if self.dbg.halted is None:
            self.show_current()

    def show_current(self) -> None:
        h = self.dbg.hart
        ((addr, text),) = self.dbg.disassemble(h.pc, 1)
        self.say(f"=> {self.dbg.where(addr)}  [ring {h.ring:+d}]  {text}")

    @staticmethod
    def _count(arg: str, default: int = 1) -> int:
        if not arg.strip():
            return default
        if not arg.strip().isdigit():
            raise UsageError(f"expected a count, got {arg.strip()!r}")
        return int(arg)

    # --- run control -----------------------------------------------------------

    def do_step(self, arg: str) -> None:
        """step [N]: run N instructions (default 1). Alias: s"""
        self.report(self.dbg.step(self._count(arg)))

    def do_next(self, _arg: str) -> None:
        """next: step, running a whole call as one step. Alias: n"""
        self.report(self.dbg.next())

    def do_continue(self, _arg: str) -> None:
        """continue: run to a breakpoint, watchpoint, caught trap or power-off.

        Alias: c"""
        self.report(self.dbg.cont())

    def do_restart(self, _arg: str) -> None:
        """restart: reload the program on a fresh machine; breakpoints stay."""
        self.dbg.restart()
        self.show_current()

    # --- stopping points -------------------------------------------------------

    def do_break(self, arg: str) -> None:
        """break ADDR: stop before running the instruction at ADDR. Alias: b

        ADDR is an expression: a label, a number, $ra, loop+6 and so on."""
        if not arg.strip():
            raise UsageError("break needs an address")
        bid, addr = self.dbg.add_breakpoint(arg)
        self.say(f"breakpoint {bid} at {self.dbg.where(addr)}")

    def do_watch(self, arg: str) -> None:
        """watch ADDR [r|w|rw]: stop after an access to ADDR (default w). Alias: w

        An aligned ADDR watches the word centred on it, any other one tryte.
        Addresses are virtual, as the program sees them."""
        parts = arg.split()
        if not parts:
            raise UsageError("watch needs an address")
        kinds = parts.pop() if len(parts) > 1 and parts[-1] in ("r", "w", "rw") else "w"
        wid, watch = self.dbg.add_watch(" ".join(parts), kinds)
        span = f"{watch.first}..{watch.last}"
        self.say(f"watchpoint {wid} on {span} ({kinds}), now {watch.value}")

    def do_catch(self, arg: str) -> None:
        """catch [all|off|CAUSE ...]: stop when a trap is taken.

        CAUSE is a number such as 4 or -1, or one of: illegal, misaligned,
        access, syscall, breakpoint, fetch-fault, load-fault, store-fault,
        timer."""
        words = arg.split()
        if not words or words == ["all"]:
            self.dbg.catch = set()
            self.say("catching every trap")
        elif words == ["off"]:
            self.dbg.catch = None
            self.say("not catching traps")
        else:
            causes = {CAUSES[w] if w in CAUSES else int(w) for w in words}
            self.dbg.catch = (self.dbg.catch or set()) | causes
            self.say(f"catching trap causes {sorted(self.dbg.catch)}")

    def do_delete(self, arg: str) -> None:
        """delete [ID]: remove a breakpoint or watchpoint, or all of them. Alias: d"""
        self.dbg.delete(self._count(arg) if arg.strip() else None)

    def do_info(self, _arg: str) -> None:
        """info: list breakpoints, watchpoints and caught traps. Alias: i"""
        for bid, addr in sorted(self.dbg.breakpoints.items()):
            if bid:
                self.say(f"  {bid}: break at {self.dbg.where(addr)}")
        for wid, w in sorted(self.dbg.watches.items()):
            self.say(f"  {wid}: watch {w.first}..{w.last} ({w.kinds})")
        catch = self.dbg.catch
        self.say(f"  catch: {'off' if catch is None else sorted(catch) or 'all'}")

    # --- inspection ------------------------------------------------------------

    def do_regs(self, _arg: str) -> None:
        """regs: show every register, the pc and the ring."""
        h = self.dbg.hart
        # Arguments and temporaries, then saved registers, then gp, tp, ra, sp.
        order = [*range(1, 14), *range(-1, -14, -1)]
        cells = [f"{isa.ABI_NAMES.get(r, f'r{r}'):>4} {h.x(r):>15}" for r in order]
        for i in range(0, len(cells), 3):
            self.say("   ".join(cells[i : i + 3]))
        self.say(f"  pc {self.dbg.where(h.pc)}   ring {h.ring:+d}")

    def do_csr(self, arg: str) -> None:
        """csr NAME|NUMBER: show a CSR, read as ring +1 would."""
        name = arg.strip().lower()
        if not name:
            raise UsageError("csr needs a name, such as mtcause")
        number = isa.CSRS.get(name)
        if number is None:
            number = self.dbg.evaluate(name)
        h = self.dbg.hart
        ring, h.ring = h.ring, 1  # look without being stopped by privilege
        try:
            value = h.read_csr(number)
        except Trap:
            raise UsageError(f"no CSR {number}") from None
        finally:
            h.ring = ring
        self.say(
            f"{isa.CSR_NAMES.get(number, number)} = {value}  0t{format_trits(value)}"
        )

    def do_print(self, arg: str) -> None:
        """print EXPR: evaluate, with $name for registers. Alias: p"""
        value = self.dbg.evaluate(arg)
        self.say(f"{value}  0t{format_trits(value)}")

    def do_examine(self, arg: str) -> None:
        """examine ADDR [N]: show N words from ADDR (default 4). Alias: x"""
        expr, count = self._addr_and_count(arg, 4)
        addr = self.dbg.evaluate(expr)
        for i in range(count):
            a = addr + 3 * i
            word = self.dbg.peek(a, 3)
            shown = (
                "<unmapped>" if word is None else f"{word:>15}  0t{format_trits(word)}"
            )
            self.say(f"{self.dbg.where(a):>24}  {shown}")

    def do_xp(self, arg: str) -> None:
        """xp ADDR [N]: show N words of physical memory from ADDR (default 4).

        Unlike examine, ADDR is not translated: it reads RAM and the boot
        page as they are, whatever ring and page table are current."""
        expr, count = self._addr_and_count(arg, 4)
        addr = self.dbg.evaluate(expr)
        for i in range(count):
            a = addr + 3 * i
            word = self.dbg.machine.bus.peek(a, 3)
            shown = (
                "<unmapped>" if word is None else f"{word:>15}  0t{format_trits(word)}"
            )
            self.say(f"{a:>24}  {shown}")

    def do_disas(self, arg: str) -> None:
        """disas [ADDR] [N]: disassemble N words from ADDR (default: the pc, 8)."""
        expr, count = self._addr_and_count(arg, 8)
        addr = self.dbg.evaluate(expr) if expr else self.dbg.hart.pc
        for a, text in self.dbg.disassemble(addr, count):
            mark = "=>" if a == self.dbg.hart.pc else "  "
            self.say(f"{mark} {self.dbg.where(a):>24}  {text}")

    def _addr_and_count(self, arg: str, default: int) -> tuple[str, int]:
        parts = arg.split()
        if len(parts) > 1 and parts[-1].isdigit():
            return " ".join(parts[:-1]), int(parts[-1])
        return " ".join(parts), default

    def do_trace(self, arg: str) -> None:
        """trace on|off: print every instruction as it runs."""
        if arg.strip() == "on":

            def show(_hart: object, _d: isa.Decoded) -> None:
                h = self.dbg.hart
                ((_, text),) = self.dbg.disassemble(h.pc, 1)
                self.say(f"   {self.dbg.where(h.pc):>24}  {text}")

            self.dbg.hart.trace = show
        elif arg.strip() == "off":
            self.dbg.hart.trace = None
        else:
            raise UsageError("trace on or trace off")

    def do_quit(self, _arg: str) -> bool:
        """quit: leave the debugger. Alias: q"""
        return True

    do_EOF = do_quit  # noqa: N815 - the name cmd.Cmd looks for at end of input
