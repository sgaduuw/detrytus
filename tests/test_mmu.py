"""Paged virtual memory (docs/isa.md section 9)."""

import pytest

from detrytus import isa
from detrytus.asm import assemble
from detrytus.machine import Machine
from detrytus.mmu import Ptbr, page_number
from detrytus.trits import field, pow3

PAGE = pow3(9)
CODE = 10  # physical page holding the program under test
FLAG_TRITS = {"r": 1, "w": 2, "x": 3, "u": 4, "g": 5, "a": 6, "d": 7}


def pte(ppn: int, flags: str = "", kind: int = 1) -> int:
    """A page-table entry: kind +1 leaf, -1 pointer; flags like 'rwuad'."""
    return kind + sum(pow3(FLAG_TRITS[f]) for f in flags) + ppn * PAGE


class Pages:
    """Builds page tables in a machine's RAM, allocating table pages from
    a counter shared by every address space on that machine."""

    def __init__(self, m: Machine, start: int = 100) -> None:
        self.m = m
        self.next = start

    def alloc(self) -> int:
        self.next += 1
        return self.next

    def space(self) -> "Space":
        return Space(self)


class Space:
    def __init__(self, pages: Pages) -> None:
        self.pages = pages
        self.root = pages.alloc()

    def entry(self, va: int, level: int) -> int:
        """Physical address of the entry for va at a level, making pointer
        entries above it as needed."""
        bus = self.pages.m.bus
        table = self.root * PAGE
        for lv in (2, 1, 0):
            idx = field(va, 9 + 8 * lv, 2 if lv == 2 else 8)
            addr = table + 3 * idx
            if lv == level:
                return addr
            current = bus.read(addr, 3, "walk")
            if field(current, 0, 1) != -1:
                current = pte(self.pages.alloc(), kind=-1)
                bus.poke_word(addr, current)
            table = (current - field(current, 0, 9)) // PAGE * PAGE
        raise AssertionError(level)

    def map(self, va_page: int, pa_page: int, flags: str, level: int = 0) -> int:
        """Map a virtual page (or superpage) and return its entry address."""
        addr = self.entry(va_page * PAGE, level)
        self.pages.m.bus.poke_word(addr, pte(pa_page, flags))
        return addr

    def raw(self, va_page: int, level: int, value: int) -> None:
        self.pages.m.bus.poke_word(self.entry(va_page * PAGE, level), value)

    def ptbr(self, asid: int = 0) -> int:
        return Ptbr.encode(1, asid, self.root)


def boot(ptbr: int, entry: int, ring: int, setup: str = "") -> str:
    """Firmware: enable paging, install a handler, drop to ring and entry.

    The handler leaves mtcause in a0 and mtval in a1 and powers off with
    the cause, so a program that ends in sys exits with 4. It runs in ring
    +1, which is never translated, however the page tables look.
    """
    return f"""
        li    t0, {ptbr}
        csrw  ptbr, t0
        la    t0, handler
        csrw  mtvec, t0
        li    t0, {entry}
        csrw  mtpc, t0
        li    t0, {ring}
        csrw  mtstatus, t0
        {setup}
        tret
handler:
        csrr  a0, mtcause
        csrr  a1, mtval
        li    t6, -39366
        sw    a0, 0(t6)
"""


def run(
    m: Machine,
    space: Space,
    program: str,
    *,
    ring: int = -1,
    entry: int = CODE,
    setup: str = "",
    asid: int = 0,
) -> int:
    source = boot(space.ptbr(asid), entry * PAGE, ring, setup)
    source += f"\n        .org {CODE * PAGE}\n" + program
    m.load(assemble(source))
    return m.run(10_000).exit_code


@pytest.fixture
def m() -> Machine:
    return Machine(ram_pages=20_000, seed=1)


@pytest.fixture
def space(m: Machine) -> Space:
    s = Pages(m).space()
    s.map(CODE, CODE, "rxua")  # the program runs in ring -1 by default
    return s


def test_user_program_runs_through_its_mapping(m, space):
    assert run(m, space, "li a2, 7\nsys") == isa.SYSCALL
    assert m.hart.reg("a2") == 7


def test_loads_and_stores_reach_the_mapped_physical_page(m, space):
    space.map(50, 60, "rwuad")
    m.bus.poke_word(60 * PAGE + 3, 12345)
    program = f"""
        li    t0, {50 * PAGE}
        lw    a2, 3(t0)
        li    t1, 99
        sw    t1, 6(t0)
        sys
    """
    assert run(m, space, program) == isa.SYSCALL
    assert m.hart.reg("a2") == 12345
    assert m.bus.read(60 * PAGE + 6, 3) == 99
    assert m.bus.read(50 * PAGE + 6, 3) == 0  # not identity mapped


FAULTS = [
    # (mapping of virtual page 50, instruction, expected cause)
    (None, "lw a2, 0(t0)", isa.LOAD_PAGE_FAULT),
    (None, "sw a2, 0(t0)", isa.STORE_PAGE_FAULT),
    (None, "jalr zero, 0(t0)", isa.INSTR_PAGE_FAULT),
    ("rua", "sw a2, 0(t0)", isa.STORE_PAGE_FAULT),  # read-only
    ("rua", "amoadd a2, a2, (t0)", isa.STORE_PAGE_FAULT),  # atomics need W
    ("rwud", "lw a2, 0(t0)", isa.LOAD_PAGE_FAULT),  # A not set
    ("rwua", "sw a2, 0(t0)", isa.STORE_PAGE_FAULT),  # D not set
    ("rwad", "lw a2, 0(t0)", isa.LOAD_PAGE_FAULT),  # kernel page from user ring
    ("rua", "jalr zero, 0(t0)", isa.INSTR_PAGE_FAULT),  # not executable
    ("wuad", "lw a2, 0(t0)", isa.LOAD_PAGE_FAULT),  # not readable
]


@pytest.mark.parametrize(("flags", "instruction", "cause"), FAULTS)
def test_page_faults_report_cause_and_virtual_address(
    m, space, flags, instruction, cause
):
    if flags is not None:
        space.map(50, 60, flags)
    va = 50 * PAGE + 3
    assert run(m, space, f"li t0, {va}\n{instruction}\nsys") == cause
    assert m.hart.reg("a1") == va


def test_load_succeeds_where_only_the_dirty_trit_is_missing(m, space):
    space.map(50, 60, "rwua")
    assert run(m, space, f"li t0, {50 * PAGE}\nlw a2, 0(t0)\nsys") == isa.SYSCALL


SUA_ON = "li t1, 0t+000\ncsrw ststatus, t1"  # ring 0 bank, trit 3


def test_kernel_reads_user_pages_only_with_sua(m, space):
    space.map(CODE, CODE, "rxa")  # kernel code
    space.map(50, 60, "rwuad")
    program = f"li t0, {50 * PAGE}\nlw a2, 0(t0)\nsys"
    assert run(m, space, program, ring=0) == isa.LOAD_PAGE_FAULT


def test_kernel_with_sua_reads_user_pages(m, space):
    space.map(CODE, CODE, "rxa")
    space.map(50, 60, "rwuad")
    program = f"li t0, {50 * PAGE}\nlw a2, 0(t0)\nsys"
    assert run(m, space, program, ring=0, setup=SUA_ON) == isa.SYSCALL


def test_kernel_never_executes_user_pages_even_with_sua(m, space):
    space.map(CODE, CODE, "rxa")
    space.map(50, 60, "rxua")
    program = f"li t0, {50 * PAGE}\njalr zero, 0(t0)"
    assert run(m, space, program, ring=0, setup=SUA_ON) == isa.INSTR_PAGE_FAULT


SUPER = pow3(8)  # pages in a mid-level superpage


@pytest.mark.parametrize(
    ("ppn", "cause"), [(SUPER, isa.SYSCALL), (SUPER + 1, isa.LOAD_PAGE_FAULT)]
)
def test_superpages_must_be_aligned(m, space, ppn, cause):
    # A leaf in a mid table maps 3**8 pages: virtual pages 3**8 + k go to
    # physical pages ppn + k, provided ppn is a multiple of 3**8.
    space.map(SUPER, ppn, "rwuad", level=1)
    m.bus.poke_word((ppn + 5) * PAGE + 3, 777)
    va = (SUPER + 5) * PAGE + 3
    assert run(m, space, f"li t0, {va}\nlw a2, 0(t0)\nsys") == cause
    if cause == isa.SYSCALL:
        assert m.hart.reg("a2") == 777


def _load_from(m: Machine, space: Space, va_page: int) -> int:
    return run(m, space, f"li t0, {va_page * PAGE}\nlw a2, 0(t0)\nsys")


def test_a_valid_mapping_loads(m, space):
    # The precondition for the reserved-entry tests below: each starts from
    # this mapping, which works, and spoils exactly one trit of it.
    space.map(SUPER + 50, 60, "rwuad")
    assert _load_from(m, space, SUPER + 50) == isa.SYSCALL


def test_a_pointer_at_leaf_level_faults(m, space):
    space.raw(SUPER + 50, 0, pte(60, "", kind=-1))
    assert _load_from(m, space, SUPER + 50) == isa.LOAD_PAGE_FAULT


def test_a_reserved_flag_value_faults(m, space):
    # X = '-' on a page the load does not need to execute: only the
    # reserved value can make this fault.
    addr = space.map(SUPER + 50, 60, "rwuad")
    m.bus.poke_word(addr, m.bus.read(addr, 3) - pow3(3))
    assert _load_from(m, space, SUPER + 50) == isa.LOAD_PAGE_FAULT
    assert m.hart.reg("a1") == (SUPER + 50) * PAGE


def test_a_pointer_carrying_permissions_faults(m, space):
    # A working mapping whose mid-level pointer also has R set. The page is
    # under its own mid entry so the code page's walk is unaffected.
    space.map(SUPER + 50, 60, "rwuad")
    addr = space.entry((SUPER + 50) * PAGE, 1)
    m.bus.poke_word(addr, m.bus.read(addr, 3) + pow3(1))
    assert _load_from(m, space, SUPER + 50) == isa.LOAD_PAGE_FAULT


def test_the_os_trit_is_free(m, space):
    space.map(50, 60, "rwuad")
    addr = space.entry(50 * PAGE, 0)
    m.bus.poke_word(addr, m.bus.read(addr, 3) - pow3(8))  # S = '-'
    assert run(m, space, f"li t0, {50 * PAGE}\nlw a2, 0(t0)\nsys") == isa.SYSCALL


def test_tables_in_device_memory_are_an_access_fault(m):
    space = Pages(m).space()
    space.root = -1  # the console's page
    assert run(m, space, "sys") == isa.ACCESS_FAULT


def test_a_stale_translation_survives_until_fence_vm(m, space):
    space.map(CODE, CODE, "rxa")
    entry = space.map(50, 60, "rwad")
    leaf_table = page_number(entry)
    space.map(leaf_table, leaf_table, "rwad")  # let the kernel edit its tables
    m.bus.poke_word(60 * PAGE, 1)
    m.bus.poke_word(61 * PAGE, 2)
    program = f"""
        li    t0, {50 * PAGE}
        lw    a2, 0(t0)
        li    t1, {entry}
        li    t2, {pte(61, "rwad")}
        sw    t2, 0(t1)          # remap page 50 to physical page 61
        lw    a3, 0(t0)          # the TLB still holds the old translation
        fence.vm t0, zero
        lw    a4, 0(t0)
        sys
    """
    assert run(m, space, program, ring=0) == isa.SYSCALL
    assert (m.hart.reg("a2"), m.hart.reg("a3"), m.hart.reg("a4")) == (1, 1, 2)


def test_asids_keep_translations_apart_and_global_entries_survive(m):
    pages = Pages(m)
    first, second = pages.space(), pages.space()
    for s in (first, second):
        s.map(CODE, CODE, "rxa")
    first.map(50, 60, "rwad")
    first.map(51, 60, "rwagd")  # global
    second.map(50, 61, "rwad")
    second.map(51, 61, "rwad")
    m.bus.poke_word(60 * PAGE, 1)
    m.bus.poke_word(61 * PAGE, 2)
    program = f"""
        li    t0, {50 * PAGE}
        li    t1, {51 * PAGE}
        lw    a2, 0(t0)          # address space 1: physical 60
        lw    a3, 0(t1)          # global entry, physical 60
        li    t2, {second.ptbr(asid=2)}
        csrw  ptbr, t2
        lw    a4, 0(t0)          # address space 2: its own translation, 61
        lw    a5, 0(t1)          # the global entry still applies: 60
        li    t3, 2
        fence.vm zero, t3        # flush address space 2: globals stay
        lw    s0, 0(t1)
        fence.vm zero, zero      # flush everything
        lw    s1, 0(t1)
        sys
    """
    assert run(m, first, program, ring=0, asid=1) == isa.SYSCALL
    got = [m.hart.reg(r) for r in ("a2", "a3", "a4", "a5", "s0", "s1")]
    assert got == [1, 1, 2, 1, 1, 2]


def test_reserved_ptbr_mode_is_not_stored(m):
    number = isa.CSRS["ptbr"]
    m.hart.write_csr(number, Ptbr.encode(1, 3, 7))
    m.hart.write_csr(number, Ptbr.encode(-1, 0, 5))
    assert m.hart.read_csr(number) == Ptbr.encode(1, 3, 7)


def test_ring_plus_one_is_never_translated(m):
    # Paging on, with an empty root table: firmware still runs.
    space = Pages(m).space()
    source = f"""
        li    t0, {space.ptbr()}
        csrw  ptbr, t0
        lt    a0, msg(zero)
        li    t6, -39366
        sw    a0, 0(t6)
msg:    .tryte 42
    """
    m.load(assemble(source))
    assert m.run(100).exit_code == 42


def test_paging_example_runs_user_code_through_its_page_table():
    from pathlib import Path

    from tests.helpers import run as run_source

    src = (Path(__file__).parent.parent / "examples" / "paging.s").read_text()
    machine, code = run_source(src)
    assert code == 0
    assert machine.console.text == (
        "user: hello from virtual page 1\nfirmware: system call from ring -1\n"
    )


# --- the checked-translation cache ---------------------------------------------
# Translations are cached per (page, access, ring, SUA) once checked. These
# make sure a check done for one of those never answers for another.


def test_a_cached_sua_read_does_not_survive_turning_sua_off(m, space):
    space.map(CODE, CODE, "rxa")
    space.map(50, 60, "rwuad")
    program = f"""
        li    t0, {50 * PAGE}
        lw    a2, 0(t0)               # allowed: SUA is on
        li    t1, 0t-000              # ststatus with SUA = -: off
        csrw  ststatus, t1
        lw    a3, 0(t0)               # the same page, now refused
        sys
    """
    assert run(m, space, program, ring=0, setup=SUA_ON) == isa.LOAD_PAGE_FAULT


def test_a_cached_load_does_not_let_a_store_through(m, space):
    space.map(50, 60, "ruad")  # readable, not writable
    program = f"""
        li    t0, {50 * PAGE}
        lw    a2, 0(t0)
        sw    a2, 0(t0)
        sys
    """
    assert run(m, space, program) == isa.STORE_PAGE_FAULT


def test_a_cached_kernel_read_does_not_answer_for_the_user_ring(m, space):
    # Ring 0 reads a kernel page (cached), then drops to ring -1, running
    # from a user page, and reads the same page, which it must not.
    space.map(CODE, CODE, "rxa")
    space.map(51, 51, "rxua")
    space.map(50, 60, "rwad")  # kernel only
    m.bus.poke_word(60 * PAGE, 5)
    program = f"""
        li    t0, {50 * PAGE}
        lw    a2, 0(t0)               # ring 0: allowed
        li    t1, {51 * PAGE}
        csrw  stpc, t1
        li    t1, -1
        csrw  ststatus, t1            # PR = -1
        tret
        .org  {51 * PAGE}
        lw    a3, 0(t0)               # ring -1: refused
        sys
    """
    assert run(m, space, program, ring=0) == isa.LOAD_PAGE_FAULT
    assert m.hart.reg("a2") == 5  # precondition: the ring 0 read happened
    assert m.hart.ring == 1  # and the fault came from user mode, into the handler
