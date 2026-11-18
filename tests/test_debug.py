"""The debugger engine and shell."""

import io
from pathlib import Path

import pytest

from detrytus import isa
from detrytus.asm import assemble
from detrytus.cli import main
from detrytus.debug import Debugger, DebugShell

EXAMPLES = Path(__file__).parent.parent / "examples"

PROGRAM = """
        .equ  POWER_OFF, -39366
        .equ  DATA, 19683           # centre of RAM page 1
start:  li    s0, DATA
        li    a0, 5
        call  double
        sw    a0, 0(s0)             # store 10
        lw    a1, 0(s0)
        sys
after:  li    t0, POWER_OFF
        sw    a1, 0(t0)
double: add   a0, a0, a0
        ret
handler:
        csrr  t1, mtpc
        addi  t1, t1, 3
        csrw  mtpc, t1
        tret
"""

# Install the handler first so the program's sys returns, then run PROGRAM.
SOURCE = (
    """
        la    t0, handler
        csrw  mtvec, t0
"""
    + PROGRAM
)


@pytest.fixture
def dbg() -> Debugger:
    return Debugger(assemble(SOURCE))


def label(dbg: Debugger, name: str) -> int:
    return dbg.image.labels[name]


def test_breakpoint_stops_before_the_instruction(dbg):
    dbg.add_breakpoint("double")
    stop = dbg.cont()
    assert stop.reason == "breakpoint"
    assert dbg.hart.pc == label(dbg, "double")
    assert dbg.hart.reg("a0") == 5  # the add has not run


def test_continue_from_a_breakpoint_moves_on(dbg):
    dbg.add_breakpoint("double")
    dbg.cont()
    stop = dbg.cont()
    assert stop.reason == "halted"
    assert stop.exit_code == 10


def test_step_runs_exactly_n_instructions(dbg):
    before = dbg.hart.instret
    assert dbg.step(3).reason == "step"
    assert dbg.hart.instret == before + 3


def test_next_runs_a_call_as_one_step(dbg):
    dbg.add_breakpoint("start+6")  # the call (li s0 and li a0 are one word each)
    dbg.cont()
    assert dbg._decoded(dbg.hart.pc).op.name == "jal"  # precondition
    call_at = dbg.hart.pc
    assert dbg.next().reason == "step"
    assert dbg.hart.pc == call_at + 3
    assert dbg.hart.reg("a0") == 10  # the callee ran
    assert 0 not in dbg.breakpoints  # the temporary breakpoint is gone


def test_next_stops_inside_a_call_at_a_breakpoint(dbg):
    dbg.add_breakpoint("start+6")
    dbg.cont()
    dbg.add_breakpoint("double")
    stop = dbg.next()
    assert stop.reason == "breakpoint"
    assert dbg.hart.pc == label(dbg, "double")


def test_write_watchpoint_reports_old_and_new(dbg):
    wid, _ = dbg.add_watch("DATA")
    stop = dbg.cont()
    assert stop.reason == "watchpoint"
    assert stop.message.startswith(f"watchpoint {wid}: write")
    assert stop.message.endswith("0 -> 10")
    # It stops after the store: the next instruction is the load.
    assert dbg._decoded(dbg.hart.pc).op.name == "lw"


def test_read_watchpoint_ignores_writes(dbg):
    dbg.add_watch("DATA", "r")
    stop = dbg.cont()
    assert stop.reason == "watchpoint"
    assert ": read at" in stop.message
    assert dbg.hart.reg("a1") == 10  # the load completed


def test_tryte_watch_does_not_fire_on_a_neighbouring_tryte(dbg):
    dbg.add_watch("DATA + 2")  # just outside the word centred on DATA
    assert dbg.cont().reason == "halted"


def test_catch_stops_on_the_chosen_trap_only(dbg):
    dbg.catch = {isa.ILLEGAL}
    assert dbg.cont().reason == "halted"
    dbg.restart()
    dbg.catch = {isa.SYSCALL}
    stop = dbg.cont()
    assert stop.reason == "trap"
    assert "cause +4" in stop.message
    assert dbg.hart.pc == label(dbg, "handler")


def test_restart_keeps_breakpoints_and_resets_the_machine(dbg):
    dbg.add_breakpoint("double")
    dbg.cont()
    dbg.cont()
    assert dbg.halted == 10
    dbg.restart()
    assert dbg.halted is None
    assert dbg.cont().reason == "breakpoint"


def test_stepping_after_power_off_says_so(dbg):
    dbg.cont()
    assert dbg.step().reason == "halted"


def test_expressions_read_registers_and_symbols(dbg):
    dbg.add_breakpoint("double")
    dbg.cont()
    assert dbg.evaluate("$a0 * 2 + DATA") == 10 + 19683
    assert dbg.evaluate("$pc") == label(dbg, "double")
    with pytest.raises(ValueError, match="no register"):
        dbg.evaluate("$nope")


def test_where_names_labels_not_constants(dbg):
    # POWER_OFF and DATA are .equ constants and must never label an address.
    assert (
        dbg.where(label(dbg, "double") + 3) == f"{label(dbg, 'double') + 3} <double+3>"
    )
    assert "DATA" not in dbg.where(19683)


def test_peek_has_no_side_effects_on_devices():
    dbg = Debugger(assemble("nop"))
    dbg.machine.console.feed("x")
    assert dbg.peek(-19680, 3) is None  # console input register
    assert list(dbg.machine.console.input) == [ord("x")]  # not consumed


def test_inspection_follows_the_programs_page_table():
    src = (EXAMPLES / "paging.s").read_text()
    dbg = Debugger(assemble(src))
    # The user program is assembled at physical page 5 and runs at virtual
    # page 1, so its labels are 4 pages above the addresses it runs at.
    dbg.add_breakpoint("next - 4 * 19683")
    assert dbg.cont().reason == "breakpoint"
    assert dbg.hart.ring == -1  # precondition: translated, in user code
    tlb_before = dict(dbg.hart.mmu.tlb)
    assert dbg.peek(2 * 19683, 3) is None  # the console: a device, not read
    assert dbg.peek(19683, 3) == dbg.machine.bus.read(5 * 19683, 3)
    assert dbg.peek(3 * 19683, 3) is None  # unmapped virtual page
    assert dbg.hart.mmu.tlb == tlb_before  # peeking cached nothing


def shell_output(commands: str, source: str = SOURCE) -> str:
    out = io.StringIO()
    shell = DebugShell(
        Debugger(assemble(source)), stdin=io.StringIO(commands), stdout=out
    )
    shell.cmdloop()
    return out.getvalue()


def test_shell_session():
    text = shell_output("b double\nc\nregs\np $a0\nx DATA 1\ndisas $pc 2\ninfo\nc\nq\n")
    assert "breakpoint 1 at" in text
    assert "stopped: breakpoint 1 at" in text
    assert "  a0               5" in text
    assert "5  0t" in text
    assert "<double>" in text
    assert "1: break at" in text
    assert "powered off with exit code 10" in text


def test_shell_reports_errors_and_keeps_going():
    text = shell_output("frob\nb\np 1 +\nx nowhere\np 2\nq\n")
    assert "unknown command 'frob'" in text
    assert "error: break needs an address" in text
    assert "error: " in text.split("break needs")[1]
    assert "2  0t" in text  # still running after the errors


def test_shell_catch_and_csr():
    text = shell_output("catch syscall\nc\ncsr mtcause\ncatch off\ni\nq\n")
    assert "catching trap causes [4]" in text
    assert "trap cause +4" in text
    assert "mtcause = 4" in text
    assert "catch: off" in text


def test_cli_batch_mode(tmp_path, capsys):
    cmds = tmp_path / "cmds"
    cmds.write_text("c\n")
    src = tmp_path / "prog.s"
    src.write_text(SOURCE)
    assert main(["debug", str(src), "-b", "double", "-x", str(cmds), "--batch"]) == 0
    out = capsys.readouterr().out
    assert "breakpoint 1 at" in out
    assert "stopped: breakpoint 1" in out
