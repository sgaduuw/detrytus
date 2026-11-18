import pytest

from detrytus import isa as I
from detrytus.cpu import UnhandledTrap
from detrytus.trits import WORD_MAX, parse_trits, wrap
from tests.helpers import HALT, result_of, run


@pytest.mark.parametrize(
    ("body", "expected"),
    [
        ("li a1, 7\nli a2, -3\nadd a0, a1, a2", 4),
        ("li a1, 7\nli a2, -3\nsub a0, a1, a2", 10),
        ("li a1, 7\nli a2, -3\nmul a0, a1, a2", -21),
        ("li a1, -7\nli a2, 2\ndiv a0, a1, a2", -3),
        ("li a1, -7\nli a2, 2\nrem a0, a1, a2", -1),
        ("li a1, 8\nli a2, 3\ndivr a0, a1, a2", 3),
        ("li a1, 8\nli a2, 3\nremr a0, a1, a2", -1),
        ("li a1, 5\nli a2, 9\ncmp a0, a1, a2", -1),
        ("li a1, 5\nli a2, 9\nmax a0, a1, a2", 9),
        ("li a1, -5\nabs a0, a1", 5),
        ("li a1, 1\nshi a0, a1, 3", 27),
        ("li a1, 41\nshi a0, a1, -1", 14),  # 41/3 = 13.67 rounds to 14
        ("li a1, 54\ntzt a0, a1", 3),
        ("li a1, 0t+0-0+\nnzt a0, a1", 3),
        ("li a1, 0t+-0+\ntex a0, a1, 1, 2", parse_trits("-0")),
        ("li a0, 0t++++\nli a1, 0t-\ntins a0, a1, 2, 1", parse_trits("+-++")),
        ("li a1, 6\nli a2, 3\nbxor a0, a1, a2", 5),
        ("lui a0, 1", 3**8),
        ("li a1, 7\nnot a0, a1", -7),
    ],
)
def test_integer_instructions(body, expected):
    assert result_of(body) == expected


def test_cmp_does_not_overflow_at_the_extremes():
    body = f"li a1, {WORD_MAX}\nli a2, {-WORD_MAX}\ncmp a0, a1, a2"
    assert result_of(body) == 1


def test_add_wraps_symmetrically():
    assert result_of(f"li a1, {WORD_MAX}\naddi a0, a1, 1") == -WORD_MAX


def test_mulh_gives_the_high_word():
    body = f"li a1, {WORD_MAX}\nli a2, {WORD_MAX - 5}\nmulh a0, a1, a2"
    product = WORD_MAX * (WORD_MAX - 5)
    expected_high = (product - wrap(product)) // 3**27
    assert expected_high != 0
    assert result_of(body) == expected_high


def test_sel_picks_by_sign():
    body = """
        li a1, 10
        li a2, 20
        li t0, -5
        sel a3, t0, a1, a2     # negative: a1
        li t0, 5
        sel a4, t0, a1, a2     # positive: a2
        sel a5, zero, a1, a2   # zero: 0
        add a0, a3, a4
        add a0, a0, a5
    """
    assert result_of(body) == 30


def test_tlut_applies_kleene_and_per_trit():
    body = "li a1, 0t+0-+\nli a2, 0t+++-\ntmin a0, a1, a2"
    assert result_of(body) == parse_trits("+0--")


@pytest.mark.parametrize(("value", "expected"), [(-4, 1), (0, 2), (9, 3)])
def test_br3_goes_three_ways(value, expected):
    body = f"""
        li    t0, {value}
        br3   t0, neg, pos
        li    a0, 2
        j     out
neg:    li    a0, 1
        j     out
pos:    li    a0, 3
out:
    """
    assert result_of(body) == expected


@pytest.mark.parametrize(
    ("cond", "a", "b", "taken"),
    [
        ("eq", 1, 1, 1),
        ("ne", 1, 1, 0),
        ("lt", -2, 1, 1),
        ("ge", 1, 1, 1),
        ("le", 2, 1, 0),
        ("gt", 2, 1, 1),
    ],
)
def test_conditional_branches(cond, a, b, taken):
    body = f"""
        li a1, {a}
        li a2, {b}
        li a0, 0
        b{cond} a1, a2, yes
        j out
yes:    li a0, 1
out:
    """
    assert result_of(body) == taken


def test_call_and_return():
    body = """
        li    a0, 5
        call  double
        call  double
        j     out
double: add   a0, a0, a0
        ret
out:
    """
    assert result_of(body) == 20


def test_word_and_tryte_memory_in_ram():
    body = """
        li    t0, 3**9 * 2       # centre of RAM page 2
        li    t1, 123456789
        sw    t1, 3(t0)
        lw    a1, 3(t0)
        li    t2, -77
        st    t2, 4(t0)          # overwrite the middle tryte of that word
        lt    a2, 4(t0)
        sub   a0, a1, a1
        add   a0, a0, a2
    """
    assert result_of(body.replace("3**9 * 2", str(3**9 * 2))) == -77


def test_word_layout_is_centred_low_trits_first():
    value = 3 + (-7) * 3**9 + 5 * 3**18  # trytes high to low: 5, -7, 3
    m, _ = run(f"li t0, {3**9}\nli t1, {value}\nsw t1, 0(t0)" + HALT)
    mem = m.bus.mem
    assert (mem[3**9 - 1], mem[3**9], mem[3**9 + 1]) == (3, -7, 5)


def test_atomics():
    body = f"""
        li    t0, {3**9}
        li    t1, 10
        sw    t1, 0(t0)
        li    t2, 5
        amoadd a1, t2, (t0)      # a1 = 10, mem = 15
        li    a2, 15
        cas   a2, t1, (t0)       # expected 15 matches: mem = 10, a2 = 15
        lr    a3, (t0)           # a3 = 10
        sc    a4, t2, (t0)       # succeeds: a4 = 0, mem = 5
        sc    a5, t2, (t0)       # no reservation: a5 = 1
        lw    t3, 0(t0)
        add   a0, a1, a2
        add   a0, a0, a3
        add   a0, a0, a4
        add   a0, a0, a5
        add   a0, a0, t3
    """
    assert result_of(body) == 10 + 15 + 10 + 0 + 1 + 5


# --- traps -------------------------------------------------------------------

HANDLER = """
        la    t0, handler
        csrw  mtvec, t0
"""


def test_unhandled_trap_stops_with_a_readable_error():
    with pytest.raises(UnhandledTrap) as e:
        run("brk")
    assert e.value.cause == I.BREAKPOINT


def test_illegal_instruction_traps_with_the_word():
    body = (
        HANDLER
        + """
        .word 0               # all-zero word: opcode 0
handler:
        csrr  a1, mtcause
        csrr  a2, mtval
        mul   a0, a1, a1
        add   a0, a0, a2
    """
    )
    assert result_of(body) == I.ILLEGAL


def test_syscall_from_user_ring_and_return():
    # Firmware drops to ring -1 with tret, the program makes a system
    # call, and the handler returns past it.
    body = (
        HANDLER
        + """
        la    t0, user
        csrw  mtpc, t0
        li    t0, -1          # tstatus.PR = -1: tret goes to user ring
        csrw  mtstatus, t0
        tret
user:
        li    a1, 41
        sys
        addi  a0, a1, 1
"""
        + HALT
        + """
handler:
        csrr  t1, mtcause     # +4: system call
        csrr  t2, mtstatus
        tex   t2, t2, 0, 1    # PR: the ring the call came from
        mul   a1, a1, t1      # 41 * 4
        add   a1, a1, t2      # -1
        csrr  t0, mtpc
        addi  t0, t0, 3
        csrw  mtpc, t0
        tret
    """
    )
    m, code = run(body)
    assert code == 41 * 4 - 1 + 1
    assert m.hart.ring == -1


def test_user_ring_cannot_touch_machine_csrs():
    body = (
        HANDLER
        + """
        la    t0, user
        csrw  mtpc, t0
        li    t0, -1
        csrw  mtstatus, t0
        tret
user:   csrr  a0, mtvec
handler:
        csrr  a0, mtcause
"""
        + HALT
    )
    _m, code = run(body)
    assert code == I.ILLEGAL


def test_misaligned_word_access_traps():
    body = (
        HANDLER + f"li t1, {3**9 + 1}\nlw a0, 0(t1)\nhandler: csrr a0, mtcause" + HALT
    )
    assert run(body)[1] == I.MISALIGNED


def test_store_to_boot_page_and_unmapped_memory_fault():
    body = (
        HANDLER
        + "sw zero, 0(zero)\nhandler: csrr a0, mtcause\ncsrr a1, mtval\nadd a0, a0, a1"
        + HALT
    )
    assert run(body)[1] == I.ACCESS_FAULT


def test_vectored_mode_dispatches_by_cause():
    body = (
        """
        la    t0, table
        addi  t0, t0, 1       # mode trit +: vectored
        csrw  mtvec, t0
        sys
        li    a0, 99
"""
        + HALT
        + """
        .align 2
        nop
table:  nop                   # cause 0 never happens
        nop                   # cause +1
        nop                   # +2
        nop                   # +3
        li    a0, 4           # +4: system call lands here
"""
        + HALT
    )
    assert run(body)[1] == 4


def test_exception_delegation_reaches_the_kernel_bank():
    body = (
        """
        la    t0, mhandler
        csrw  mtvec, t0
        la    t0, shandler
        csrw  stvec, t0
        li    t0, 0t+0000     # trit 4 (cause +4) delegated to ring 0
        csrw  edeleg, t0
        la    t0, user
        csrw  mtpc, t0
        li    t0, -1
        csrw  mtstatus, t0
        tret
user:   sys
mhandler:
        li    a0, 1
"""
        + HALT
        + """
shandler:
        li    a0, 2
"""
        + HALT
    )
    m, code = run(body)
    assert code == 2
    assert m.hart.ring == 0


def test_timer_interrupt_fires_after_timecmp():
    body = (
        """
        la    t0, handler
        csrw  mtvec, t0
        li    t0, 50
        csrw  mtimecmp, t0
        li    t0, 0t+0        # ie trit 1: timer
        csrw  mie, t0
        li    t0, 0t+0        # tstatus.IE = +
        csrw  mtstatus, t0
        li    a0, 0
spin:   addi  a0, a0, 1
        j     spin
handler:
        csrr  a1, mtcause
        csrr  a2, time
"""
        + HALT
    )
    m, code = run(body)
    assert m.hart.regs[2 + 13] == I.IRQ_TIMER  # a1
    assert m.hart.regs[3 + 13] >= 50  # a2
    assert code > 0


def test_wfi_skips_ahead_to_the_timer():
    body = (
        """
        la    t0, handler
        csrw  mtvec, t0
        li    t0, 1000000
        csrw  mtimecmp, t0
        li    t0, 0t+0
        csrw  mie, t0
        csrw  mtstatus, t0
        wfi
handler:
        csrr  a0, mcause
""".replace("mcause", "mtcause")
        + HALT
    )
    _m, code = run(body, max_steps=100)
    assert code == I.IRQ_TIMER


def test_single_step_traps_after_one_user_instruction():
    body = (
        HANDLER
        + """
        la    t0, user
        csrw  mtpc, t0
        li    t0, 0t+000-     # SS (trit 4) = +, PR = -1
        csrw  mtstatus, t0
        tret
user:   addi  a1, zero, 7
        addi  a1, a1, 100     # never reached
handler:
        csrr  a0, mtcause
        mul   a0, a0, a1      # 9 * 7
"""
        + HALT
    )
    assert run(body)[1] == I.SINGLE_STEP * 7


def test_entropy_and_counters_read():
    body = "csrr a1, entropy\ncsrr a2, instret\nmv a0, a2" + HALT
    _m, code = run(body)
    assert code == 1  # only the first instruction has retired when instret is read


def test_hello_example_prints():
    from pathlib import Path

    src = (Path(__file__).parent.parent / "examples" / "hello.s").read_text()
    m, code = run(src)
    assert code == 0
    assert m.console.text == "Hello from a balanced-ternary machine!\n"


def test_delegated_trap_raised_in_ring_plus_one_stays_there():
    # Delegation sends system calls to ring 0, but a trap never lands in a
    # ring below the one it interrupts, so firmware's own sys stays put.
    body = (
        """
        la    t0, mhandler
        csrw  mtvec, t0
        la    t0, shandler
        csrw  stvec, t0
        li    t0, 0t+0000
        csrw  edeleg, t0
        sys
mhandler:
        li    a0, 1
"""
        + HALT
        + """
shandler:
        li    a0, 2
"""
        + HALT
    )
    m, code = run(body)
    assert m.hart.banks[1][3] == I.SYSCALL  # the call reached the machine bank
    assert code == 1


def test_bsearch_example_finds_every_key():
    from pathlib import Path

    src = (Path(__file__).parent.parent / "examples" / "bsearch.s").read_text()
    m, code = run(src)
    assert code == 0
    assert m.console.text.split() == ["5", "0", "8", "-1", "2"]
