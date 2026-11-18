"""One hart: registers, CSRs, traps, interrupts and instruction execution.

Implemented: every integer, logic, memory, atomic, control and system
instruction of docs/isa.md section 7, the trap and interrupt model of
section 8 and single step. Not yet: floating point (7.11), compressed
bundles (6.1), paging (9) and debug triggers (8.6). Those raise
NotImplementedError rather than silently misbehaving.
"""

import os
from collections.abc import Callable

from detrytus import alu
from detrytus import isa as I
from detrytus.bus import Bus, BusFault
from detrytus.trits import WORD_TRITS, field, low, sign, wrap

# Bank indices (8.2)
TSTATUS, TVEC, TPC, TCAUSE, TVAL, SCRATCH, IE, IP, TIMECMP = range(9)
# tstatus trits
PR, IEN, PIE, SUA, SS = range(5)

# Interrupt priority when several are pending: external, software, timer.
IRQ_PRIORITY = (2, 3, 1)


class Trap(Exception):
    def __init__(self, cause: int, tval: int = 0) -> None:
        super().__init__(f"trap cause {cause}, tval {tval}")
        self.cause = cause
        self.tval = tval


class UnhandledTrap(Exception):
    """A trap arrived at a ring whose tvec is 0: no handler installed.

    Address 0 is the reset vector, so a handler there would mean re-running
    the firmware. The emulator treats it as "nothing installed" and stops,
    which turns a silent loop into a readable error.
    """

    def __init__(self, cause: int, tval: int, pc: int, ring: int) -> None:
        super().__init__(
            f"unhandled trap: cause {cause:+d} at pc {pc} in ring {ring:+d}, "
            f"tval {tval}"
        )
        self.cause = cause
        self.tval = tval
        self.pc = pc
        self.ring = ring


def set_trit(x: int, i: int, value: int) -> int:
    return x + (value - field(x, i, 1)) * 3**i


class Hart:
    def __init__(self, bus: Bus, hartid: int = 0, seed: int | None = None) -> None:
        self.bus = bus
        self.hartid = hartid
        self.regs = [0] * 27
        self.pc = 0
        self.ring = 1
        self.banks = {1: [0] * 9, 0: [0] * 9}
        self.csrs = {
            I.CSRS[n]: 0
            for n in (
                "edeleg",
                "ideleg",
                "ptbr",
                "fcsr",
                *(f"tdaddr{i}" for i in range(9)),
                *(f"tdctl{i}" for i in range(9)),
            )
        }
        self.instret = 0
        self.time = 0
        self.reservation: int | None = None
        self.step_ring: int | None = None  # bank to trap into after one step
        self.trace: Callable[[Hart, I.Decoded], None] | None = None
        self._entropy = (
            (lambda: int.from_bytes(os.urandom(6))) if seed is None else _seeded(seed)
        )
        self._cache: dict[int, I.Decoded | I.IllegalInstruction] = {}
        self._exec = {
            op.opcode: getattr(self, "_op_" + _ident(op.name)) for op in I.OPS.values()
        }

    # --- registers ---------------------------------------------------------

    def x(self, r: int) -> int:
        return self.regs[r + 13]

    def set_x(self, r: int, value: int) -> None:
        if r:
            self.regs[r + 13] = value

    def reg(self, name: str) -> int:
        return self.x(I.REG_NAMES[name])

    # --- running -----------------------------------------------------------

    def run(self, max_steps: int | None = None) -> int:
        """Run until the step limit; returns the number of steps taken."""
        n = 0
        while max_steps is None or n < max_steps:
            self.step()
            n += 1
        return n

    def step(self) -> None:
        self.time += 1
        stepping = self.step_ring
        self.step_ring = None
        if not self._take_interrupt():
            pc = self.pc
            try:
                self._execute(pc)
                self.instret += 1
                if stepping is not None:
                    raise Trap(I.SINGLE_STEP, self.pc)
            except Trap as t:
                self._trap(
                    t.cause,
                    t.tval,
                    self.pc if t.cause == I.SINGLE_STEP else pc,
                    min_ring=stepping if t.cause == I.SINGLE_STEP else None,
                )

    def _execute(self, pc: int) -> None:
        if low(pc, 1) != 0:
            raise NotImplementedError("compressed bundles (slot B) are not implemented")
        if self.ring <= 0 and low(self.csrs[I.CSRS["ptbr"]], 1) == 1:
            raise NotImplementedError("paged virtual memory is not implemented")
        try:
            word = self.bus.read(pc, 3, "fetch")
        except BusFault as f:
            raise Trap(I.ACCESS_FAULT, f.addr) from None
        d = self._cache.get(word)
        if d is None:
            if low(word, 5) == I.BUNDLE_OPCODE:
                raise NotImplementedError("compressed bundles are not implemented")
            try:
                d = I.decode(word)
            except I.IllegalInstruction as e:
                d = e
            self._cache[word] = d
        if isinstance(d, I.IllegalInstruction):
            raise Trap(I.ILLEGAL, word)
        if self.trace is not None:
            self.trace(self, d)
        self.next_pc = wrap(pc + 3)
        self._exec[d.op.opcode](d)
        self.pc = self.next_pc

    # --- traps and interrupts ----------------------------------------------

    def _trap(
        self, cause: int, tval: int, tpc: int, min_ring: int | None = None
    ) -> None:
        if min_ring is not None:
            target = min_ring
        elif cause > 0:
            delegated = field(self.csrs[I.CSRS["edeleg"]], cause, 1) == 1
            target = 0 if delegated else 1
        else:
            target = 1
        # A trap never lands in a ring below the one it interrupts.
        target = max(target, self.ring)
        bank = self.banks[target]
        tvec = bank[TVEC]
        if tvec == 0:
            raise UnhandledTrap(cause, tval, tpc, self.ring)
        bank[TPC] = tpc
        bank[TCAUSE] = cause
        bank[TVAL] = tval
        st = bank[TSTATUS]
        st = set_trit(st, PR, self.ring)
        st = set_trit(st, PIE, field(st, IEN, 1))
        bank[TSTATUS] = set_trit(st, IEN, -1)
        self.ring = target
        self.reservation = None
        mode = low(tvec, 1)
        base = tvec - mode
        self.pc = wrap(base + 3 * cause) if mode == 1 else base

    def _pending(self, r: int) -> int:
        """ip for ring r's bank, with the timer trit computed from time."""
        bank = self.banks[r]
        timer = 1 if self.time >= bank[TIMECMP] else 0
        return set_trit(bank[IP], 1, timer)

    def _take_interrupt(self) -> bool:
        for r in (1, 0):
            bank = self.banks[r]
            ie = bank[IE]
            if ie == 0:
                continue
            if not (
                self.ring < r or (self.ring == r and field(bank[TSTATUS], IEN, 1) == 1)
            ):
                continue
            ip = self._pending(r)
            for i in IRQ_PRIORITY:
                if field(ip, i, 1) == 1 and field(ie, i, 1) == 1:
                    self._trap(-i, 0, self.pc, min_ring=r)
                    return True
        return False

    # --- CSRs --------------------------------------------------------------

    def _bank_slot(self, number: int) -> tuple[int, int] | None:
        for r in (1, 0):
            base = I.csr_number(r, 1, 0)
            if base <= number < base + 9:
                return r, number - base
        return None

    def read_csr(self, number: int) -> int:
        if self.ring < I.csr_ring(number):
            raise Trap(I.ILLEGAL)
        slot = self._bank_slot(number)
        if slot is not None:
            r, i = slot
            return self._pending(r) if i == IP else self.banks[r][i]
        name = I.CSR_NAMES.get(number)
        if name in ("cycle", "instret"):
            return wrap(self.instret)
        if name == "time":
            return wrap(self.time)
        if name == "hartid":
            return self.hartid
        if name == "entropy":
            return wrap(self._entropy())
        if number in self.csrs:
            return self.csrs[number]
        raise Trap(I.ILLEGAL)

    def write_csr(self, number: int, value: int) -> None:
        if not I.csr_writable(number):
            raise Trap(I.ILLEGAL)
        slot = self._bank_slot(number)
        if slot is not None:
            r, i = slot
            if i == IP:
                # Only the software-interrupt trit is writable; the timer
                # trit follows timecmp and the external trit follows devices.
                value = set_trit(self.banks[r][IP], 3, field(value, 3, 1))
            self.banks[r][i] = value
            return
        self.csrs[number] = value

    def _csr_op(self, d: I.Decoded, combine: Callable[[int, int], int] | None) -> None:
        number = low(d.imm, 9)
        old = self.read_csr(number)
        if d.rs1 != 0:
            src = self.x(d.rs1)
            self.write_csr(number, src if combine is None else combine(old, src))
        self.set_x(d.rd, old)

    # --- memory helpers ----------------------------------------------------

    def load(self, addr: int, width: int) -> int:
        if width == 3 and low(addr, 1) != 0:
            raise Trap(I.MISALIGNED, addr)
        try:
            return self.bus.read(addr, width)
        except BusFault as f:
            raise Trap(I.ACCESS_FAULT, f.addr) from None

    def store(self, addr: int, width: int, value: int) -> None:
        if width == 3 and low(addr, 1) != 0:
            raise Trap(I.MISALIGNED, addr)
        try:
            self.bus.write(addr, width, value)
        except BusFault as f:
            raise Trap(I.ACCESS_FAULT, f.addr) from None

    def _amo_addr(self, d: I.Decoded) -> int:
        addr = self.x(d.rs1)
        if low(addr, 1) != 0:
            raise Trap(I.MISALIGNED, addr)
        return addr

    def _amo(self, d: I.Decoded, update: Callable[[int, int], int]) -> None:
        addr = self._amo_addr(d)
        old = self.load(addr, 3)
        self.store(addr, 3, update(old, self.x(d.rs2)))
        self.set_x(d.rd, old)

    def _privileged(self) -> None:
        if self.ring < 0:
            raise Trap(I.ILLEGAL)

    # --- instructions ------------------------------------------------------

    def _alu(self, d: I.Decoded, fn: Callable[[int, int], int]) -> None:
        self.set_x(d.rd, fn(self.x(d.rs1), self.x(d.rs2)))

    def _op_add(self, d):
        self._alu(d, lambda a, b: wrap(a + b))

    def _op_sub(self, d):
        self._alu(d, lambda a, b: wrap(a - b))

    def _op_mul(self, d):
        self._alu(d, lambda a, b: alu.mul_parts(a, b)[1])

    def _op_mulh(self, d):
        self._alu(d, lambda a, b: alu.mul_parts(a, b)[0])

    def _op_div(self, d):
        self._alu(d, lambda a, b: alu.div_trunc(a, b)[0])

    def _op_rem(self, d):
        self._alu(d, lambda a, b: alu.div_trunc(a, b)[1])

    def _op_divr(self, d):
        self._alu(d, lambda a, b: alu.div_round(a, b)[0])

    def _op_remr(self, d):
        self._alu(d, lambda a, b: alu.div_round(a, b)[1])

    def _op_cmp(self, d):
        self._alu(d, lambda a, b: sign(a - b))

    def _op_min(self, d):
        self._alu(d, min)

    def _op_max(self, d):
        self._alu(d, max)

    def _op_abs(self, d):
        self.set_x(d.rd, abs(self.x(d.rs1)))

    def _op_sel(self, d):
        c = self.x(d.rs1)
        self.set_x(d.rd, self.x(d.rs2) if c < 0 else self.x(d.rs3) if c > 0 else 0)

    def _op_sh(self, d):
        self._alu(d, alu.shift)

    def _op_rot(self, d):
        self._alu(d, alu.rotate)

    def _op_shi(self, d):
        self.set_x(d.rd, alu.shift(self.x(d.rs1), d.imm))

    def _op_roti(self, d):
        self.set_x(d.rd, alu.rotate(self.x(d.rs1), d.imm))

    def _field_args(self, d: I.Decoded) -> tuple[int, int]:
        pos, length = low(d.imm, 4), field(d.imm, 4, 4)
        if pos < 0 or length < 1 or pos + length > WORD_TRITS:
            raise Trap(I.ILLEGAL, d.word)
        return pos, length

    def _op_tex(self, d):
        pos, length = self._field_args(d)
        self.set_x(d.rd, alu.trit_extract(self.x(d.rs1), pos, length))

    def _op_tins(self, d):
        pos, length = self._field_args(d)
        self.set_x(d.rd, alu.trit_insert(self.x(d.rd), self.x(d.rs1), pos, length))

    def _op_lzt(self, d):
        self.set_x(d.rd, alu.leading_zero_trits(self.x(d.rs1)))

    def _op_tzt(self, d):
        self.set_x(d.rd, alu.trailing_zero_trits(self.x(d.rs1)))

    def _op_nzt(self, d):
        self.set_x(d.rd, alu.nonzero_trits(self.x(d.rs1)))

    def _op_tlut(self, d):
        self._alu(d, lambda a, b: alu.tlut(a, b, d.fn))

    def _op_band(self, d):
        self._alu(d, lambda a, b: wrap(a & b))

    def _op_bor(self, d):
        self._alu(d, lambda a, b: wrap(a | b))

    def _op_bxor(self, d):
        self._alu(d, lambda a, b: wrap(a ^ b))

    def _op_bsh(self, d):
        self._alu(d, alu.binary_shift)

    def _op_addi(self, d):
        self.set_x(d.rd, wrap(self.x(d.rs1) + d.imm))

    def _op_cmpi(self, d):
        self.set_x(d.rd, sign(self.x(d.rs1) - d.imm))

    def _op_lui(self, d):
        self.set_x(d.rd, d.imm * 3**8)

    def _op_auipc(self, d):
        self.set_x(d.rd, wrap(self.pc + d.imm * 3**8))

    def _op_lt(self, d):
        self.set_x(d.rd, self.load(wrap(self.x(d.rs1) + d.imm), 1))

    def _op_lw(self, d):
        self.set_x(d.rd, self.load(wrap(self.x(d.rs1) + d.imm), 3))

    def _op_st(self, d):
        self.store(wrap(self.x(d.rs1) + d.imm), 1, low(self.x(d.rs2), 9))

    def _op_sw(self, d):
        self.store(wrap(self.x(d.rs1) + d.imm), 3, self.x(d.rs2))

    def _op_lr(self, d):
        addr = self._amo_addr(d)
        self.set_x(d.rd, self.load(addr, 3))
        self.reservation = addr

    def _op_sc(self, d):
        addr = self._amo_addr(d)
        ok = self.reservation == addr
        self.reservation = None
        if ok:
            self.store(addr, 3, self.x(d.rs2))
        self.set_x(d.rd, 0 if ok else 1)

    def _op_amoswap(self, d):
        self._amo(d, lambda old, src: src)

    def _op_amoadd(self, d):
        self._amo(d, lambda old, src: wrap(old + src))

    def _op_amomin(self, d):
        self._amo(d, min)

    def _op_amomax(self, d):
        self._amo(d, max)

    def _op_amotlut(self, d):
        self._amo(d, lambda old, src: alu.tlut(old, src, d.fn))

    def _op_cas(self, d):
        addr = self._amo_addr(d)
        old = self.load(addr, 3)
        if old == self.x(d.rd):
            self.store(addr, 3, self.x(d.rs2))
        self.set_x(d.rd, old)

    def _op_jal(self, d):
        self.set_x(d.rd, self.next_pc)
        self.next_pc = wrap(self.pc + 3 * d.imm)

    def _op_jalr(self, d):
        target = wrap(self.x(d.rs1) + d.imm)
        if low(target, 1) == -1:
            raise Trap(I.MISALIGNED, target)
        self.set_x(d.rd, self.next_pc)
        self.next_pc = target

    def _op_b(self, d):
        if sign(self.x(d.rs1) - self.x(d.rs2)) in I.COND_TAKEN[d.cond]:
            self.next_pc = wrap(self.pc + 3 * d.off)

    def _op_br3(self, d):
        s = sign(self.x(d.rs1))
        if s:
            self.next_pc = wrap(self.pc + 3 * (d.offn if s < 0 else d.offp))

    def _op_sys(self, d):
        raise Trap(I.SYSCALL)

    def _op_brk(self, d):
        raise Trap(I.BREAKPOINT, self.pc)

    def _op_tret(self, d):
        self._privileged()
        bank = self.banks[self.ring]
        st = bank[TSTATUS]
        back = field(st, PR, 1)
        if back > self.ring:
            raise Trap(I.ILLEGAL, d.word)
        bank[TSTATUS] = set_trit(st, IEN, field(st, PIE, 1))
        if field(st, SS, 1) == 1:
            self.step_ring = self.ring
        self.ring = back
        self.next_pc = bank[TPC]

    def _op_csrrw(self, d):
        self._csr_op(d, None)

    def _op_csrrtmax(self, d):
        self._csr_op(d, lambda old, src: alu.tlut(old, src, I.TABLES["tmax"]))

    def _op_csrrtmin(self, d):
        self._csr_op(d, lambda old, src: alu.tlut(old, src, I.TABLES["tmin"]))

    def _op_fence(self, d):
        pass  # one hart, no store buffer: already ordered

    def _op_fence_i(self, d):
        pass  # the decode cache is keyed by word value, so it is never stale

    def _op_fence_vm(self, d):
        self._privileged()

    def _op_wfi(self, d):
        # Nothing else can happen in a lone hart, so skip ahead to the
        # earliest enabled timer instead of spinning.
        deadlines = [
            b[TIMECMP]
            for b in self.banks.values()
            if field(b[IE], 1, 1) == 1 and b[TIMECMP] > self.time
        ]
        if deadlines:
            self.time = min(deadlines)

    def _op_cclean(self, d):
        pass

    def _op_cflush(self, d):
        pass

    def _op_cinval(self, d):
        self._privileged()


def _ident(name: str) -> str:
    return name.replace(".", "_")


def _seeded(seed: int) -> Callable[[], int]:
    import random

    rng = random.Random(seed)
    return lambda: rng.getrandbits(48)
