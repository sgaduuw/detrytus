"""One hart: registers, CSRs, traps, interrupts and the fetch loop.

Implemented: every integer, logic, memory, atomic, control and system
instruction of docs/isa.md section 7 (their semantics live in
execute.py), the trap and interrupt model of section 8, single step,
and paged virtual memory (section 9, in mmu.py). Not yet: floating point
(7.11), compressed bundles (6.1) and debug triggers (8.6). Those raise
NotImplementedError rather than silently misbehaving.

Single-hart assumptions, in one place: there is no store buffer, so
fences are no-ops; nothing can wake a waiting hart except its timer, so
wfi skips time ahead; and an lr reservation is lost only to a trap or a
later sc.
"""

import os
import random
from collections.abc import Callable

from detrytus import isa
from detrytus.bus import Bus, BusFault
from detrytus.execute import BY_OPCODE
from detrytus.mmu import Access, Mmu, Ptbr
from detrytus.trap import Trap, UnhandledTrap
from detrytus.trits import field, low, set_trit, wrap

# Interrupt priority when several are pending: external, software, timer.
IRQ_PRIORITY = (2, 3, 1)

# CSRs held as plain storage; the others are computed or live in banks.
STORED_CSRS = (
    "edeleg",
    "ideleg",
    "ptbr",
    "fcsr",
    *(f"tdaddr{i}" for i in range(9)),
    *(f"tdctl{i}" for i in range(9)),
)

Tracer = Callable[["Hart", isa.Decoded], None]
# Debugger hooks: (virtual address, width, "r" | "w" | "rw") after a
# completed data access, and (cause, tval, tpc) as a trap is entered.
AccessHook = Callable[[int, int, str], None]
TrapHook = Callable[[int, int, int], None]


class Hart:
    def __init__(self, bus: Bus, hartid: int = 0, seed: int | None = None) -> None:
        self.bus = bus
        self.hartid = hartid
        self.regs = [0] * 27
        self.pc = 0
        self.next_pc = 0  # set by each instruction before it runs
        self.ring = 1
        self.banks: dict[int, list[int]] = {1: [0] * 9, 0: [0] * 9}
        self.csrs: dict[int, int] = {isa.CSRS[name]: 0 for name in STORED_CSRS}
        self.instret = 0
        self.time = 0
        self.reservation: int | None = None
        self.step_ring: int | None = None  # bank to trap into after one step
        self.trace: Tracer | None = None
        self.on_access: AccessHook | None = None
        self.on_trap: TrapHook | None = None
        self.mmu = Mmu(bus)
        self._ptbr = Ptbr.decode(0)  # ptbr, decoded once per write
        self._entropy = _entropy_source(seed)
        self._cache: dict[int, isa.Decoded | isa.IllegalInstruction] = {}

    # --- registers ---------------------------------------------------------

    def x(self, r: int) -> int:
        return self.regs[r + 13]

    def set_x(self, r: int, value: int) -> None:
        if r:
            self.regs[r + 13] = value

    def reg(self, name: str) -> int:
        """A register by any of its names: 'a0', 'r1', 'sp', ..."""
        return self.x(isa.REG_NAMES[name])

    # --- the fetch loop ----------------------------------------------------

    def step(self) -> None:
        self.time += 1
        stepping = self.step_ring
        self.step_ring = None
        if self._take_interrupt():
            return
        pc = self.pc
        try:
            self._execute(pc)
            self.instret += 1
            if stepping is not None:
                raise Trap(isa.SINGLE_STEP, self.pc)
        except Trap as t:
            if t.cause == isa.SINGLE_STEP:
                self._trap(t.cause, t.tval, self.pc, target=stepping)
            else:
                self._trap(t.cause, t.tval, pc)

    def _execute(self, pc: int) -> None:
        if low(pc, 1) != 0:
            raise NotImplementedError("compressed bundles (slot B) are not implemented")
        try:
            word = self.bus.read(self.translate(pc, Access.FETCH), 3, "fetch")
        except BusFault as f:
            raise Trap(isa.ACCESS_FAULT, f.addr) from None
        d = self._decode(word)
        if self.trace is not None:
            self.trace(self, d)
        self.next_pc = wrap(pc + 3)
        BY_OPCODE[d.op.opcode](self, d)
        self.pc = self.next_pc

    def _decode(self, word: int) -> isa.Decoded:
        d = self._cache.get(word)
        if d is None:
            if low(word, 5) == isa.BUNDLE_OPCODE:
                raise NotImplementedError("compressed bundles are not implemented")
            try:
                d = isa.decode(word)
            except isa.IllegalInstruction as e:
                d = e
            self._cache[word] = d
        if isinstance(d, isa.IllegalInstruction):
            raise Trap(isa.ILLEGAL, word)
        return d

    # --- traps and interrupts ----------------------------------------------

    def _trap(self, cause: int, tval: int, tpc: int, target: int | None = None) -> None:
        if self.on_trap is not None:
            self.on_trap(cause, tval, tpc)
        if target is None:
            delegated = (
                cause > 0 and field(self.csrs[isa.CSRS["edeleg"]], cause, 1) == 1
            )
            target = 0 if delegated else 1
        # A trap never lands in a ring below the one it interrupts.
        target = max(target, self.ring)
        bank = self.banks[target]
        tvec = bank[isa.TVEC]
        if tvec == 0:
            raise UnhandledTrap(cause, tval, tpc, self.ring)
        bank[isa.TPC] = tpc
        bank[isa.TCAUSE] = cause
        bank[isa.TVAL] = tval
        status = set_trit(bank[isa.TSTATUS], isa.ST_PR, self.ring)
        status = set_trit(status, isa.ST_PIE, field(status, isa.ST_IE, 1))
        bank[isa.TSTATUS] = set_trit(status, isa.ST_IE, -1)
        self.ring = target
        self.reservation = None
        mode = low(tvec, 1)
        base = tvec - mode
        self.pc = wrap(base + 3 * cause) if mode == 1 else base

    def _pending(self, ring: int) -> int:
        """ip for a ring's bank, with the timer trit computed from time."""
        bank = self.banks[ring]
        timer = 1 if self.time >= bank[isa.TIMECMP] else 0
        return set_trit(bank[isa.IP], 1, timer)

    def _interruptible(self, ring: int) -> bool:
        if self.ring < ring:
            return True
        status = self.banks[ring][isa.TSTATUS]
        return self.ring == ring and field(status, isa.ST_IE, 1) == 1

    def _take_interrupt(self) -> bool:
        for ring in (1, 0):
            enabled = self.banks[ring][isa.IE]
            if enabled == 0 or not self._interruptible(ring):
                continue
            pending = self._pending(ring)
            for i in IRQ_PRIORITY:
                if field(pending, i, 1) == 1 and field(enabled, i, 1) == 1:
                    self._trap(-i, 0, self.pc, target=ring)
                    return True
        return False

    # --- CSRs --------------------------------------------------------------

    @staticmethod
    def _bank_slot(number: int) -> tuple[int, int] | None:
        for ring in (1, 0):
            base = isa.csr_number(ring, 1, 0)
            if base <= number < base + len(isa.BANK):
                return ring, number - base
        return None

    def read_csr(self, number: int) -> int:
        if self.ring < isa.csr_ring(number):
            raise Trap(isa.ILLEGAL)
        slot = self._bank_slot(number)
        if slot is not None:
            ring, i = slot
            return self._pending(ring) if i == isa.IP else self.banks[ring][i]
        computed = {
            "cycle": lambda: self.instret,
            "instret": lambda: self.instret,
            "time": lambda: self.time,
            "hartid": lambda: self.hartid,
            "entropy": self._entropy,
        }.get(isa.CSR_NAMES.get(number, ""))
        if computed is not None:
            return wrap(computed())
        if number in self.csrs:
            return self.csrs[number]
        raise Trap(isa.ILLEGAL)

    def write_csr(self, number: int, value: int) -> None:
        if not isa.csr_writable(number):
            raise Trap(isa.ILLEGAL)
        slot = self._bank_slot(number)
        if slot is None:
            if number == isa.CSRS["ptbr"]:
                if low(value, 1) == -1:
                    return  # a reserved mode is not stored (9.2)
                self._ptbr = Ptbr.decode(value)
            self.csrs[number] = value
            return
        ring, i = slot
        if i == isa.IP:
            # Only the software-interrupt trit is writable; the timer
            # trit follows timecmp and the external trit follows devices.
            value = set_trit(self.banks[ring][isa.IP], 3, field(value, 3, 1))
        self.banks[ring][i] = value

    # --- memory ---------------------------------------------------------

    def translate(self, addr: int, access: Access) -> int:
        """Virtual to physical. Ring +1 is never translated (8.1)."""
        if self.ring > 0:
            return addr
        ptbr = self._ptbr
        if ptbr.mode != 1:
            return addr
        status = self.banks[0][isa.TSTATUS]
        sua = field(status, isa.ST_SUA, 1) == 1
        return self.mmu.translate(addr, access, self.ring, sua, ptbr)

    def probe(self, addr: int) -> int | None:
        """Where addr points for the current ring, without side effects.

        For debuggers: no permission check, nothing cached in the TLB, and
        None where nothing is mapped.
        """
        if self.ring > 0:
            return addr
        ptbr = self._ptbr
        return addr if ptbr.mode != 1 else self.mmu.probe(addr, ptbr)

    @staticmethod
    def _aligned(addr: int, width: int) -> None:
        if width == 3 and low(addr, 1) != 0:
            raise Trap(isa.MISALIGNED, addr)

    def load(self, addr: int, width: int) -> int:
        self._aligned(addr, width)
        physical = self.translate(addr, Access.LOAD)
        try:
            value = self.bus.read(physical, width)
        except BusFault:
            raise Trap(isa.ACCESS_FAULT, addr) from None
        if self.on_access is not None:
            self.on_access(addr, width, "r")
        return value

    def store(self, addr: int, width: int, value: int) -> None:
        self._aligned(addr, width)
        physical = self.translate(addr, Access.STORE)
        try:
            self.bus.write(physical, width, value)
        except BusFault:
            raise Trap(isa.ACCESS_FAULT, addr) from None
        if self.on_access is not None:
            self.on_access(addr, width, "w")

    def rmw(self, addr: int, update: Callable[[int], int]) -> int:
        """Atomic read-modify-write of a word; returns the old value.

        One translation covers both halves and needs read and write
        permission, so a read-only page faults as a store would.
        """
        self._aligned(addr, 3)
        physical = self.translate(addr, Access.AMO)
        try:
            old = self.bus.read(physical, 3)
            self.bus.write(physical, 3, update(old))
        except BusFault:
            raise Trap(isa.ACCESS_FAULT, addr) from None
        if self.on_access is not None:
            self.on_access(addr, 3, "rw")
        return old


def _entropy_source(seed: int | None) -> Callable[[], int]:
    """Hardware randomness, or a reproducible stream when seeded for tests."""
    if seed is None:
        return lambda: int.from_bytes(os.urandom(6))
    rng = random.Random(seed)
    return lambda: rng.getrandbits(48)
