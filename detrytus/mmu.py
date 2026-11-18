"""Paged virtual memory: the page walk, permission checks and the TLB.

The rules are docs/isa.md section 9. The TLB here keeps every successful
translation until fence.vm removes it and never evicts on its own. That
is the strictest behaviour the spec allows, which is the point: a
kernel that forgets a fence.vm sees stale translations at once instead
of on some later, larger machine.
"""

from dataclasses import dataclass
from enum import Enum

from detrytus import isa
from detrytus.bus import PAGE_TRITS, Bus, BusFault
from detrytus.trap import Trap
from detrytus.trits import field, low, pow3, split_high

LEVEL_TRITS = 8  # index width of mid and leaf tables
ROOT_LEVEL = 2

# PTE trits (9.3)
V, R, W, X, U, G, A, D, S = range(9)


class Access(Enum):
    FETCH = "fetch"
    LOAD = "load"
    STORE = "store"
    AMO = "amo"  # read-modify-write: needs R and W, faults like a store


FAULT_CAUSE = {
    Access.FETCH: isa.INSTR_PAGE_FAULT,
    Access.LOAD: isa.LOAD_PAGE_FAULT,
    Access.STORE: isa.STORE_PAGE_FAULT,
    Access.AMO: isa.STORE_PAGE_FAULT,
}

NEEDS: dict[Access, tuple[int, ...]] = {
    Access.FETCH: (X,),
    Access.LOAD: (R,),
    Access.STORE: (W,),
    Access.AMO: (R, W),
}


@dataclass(frozen=True)
class Ptbr:
    mode: int
    asid: int
    root: int  # physical page whose centre holds the root table

    @classmethod
    def decode(cls, value: int) -> "Ptbr":
        return cls(low(value, 1), field(value, 1, 8), split_high(value, PAGE_TRITS))

    @staticmethod
    def encode(mode: int, asid: int, root: int) -> int:
        return mode + 3 * asid + root * pow3(PAGE_TRITS)


@dataclass(frozen=True)
class Leaf:
    """A cached translation: the leaf PTE and the physical centre of the
    base page the virtual page maps to."""

    pte: int
    page_centre: int


def page_number(addr: int) -> int:
    return split_high(addr, PAGE_TRITS)


def index(va: int, level: int) -> int:
    """The table index for a level: root trits 26..25, mid 24..17, leaf 16..9."""
    lo = PAGE_TRITS + LEVEL_TRITS * level
    width = 2 if level == ROOT_LEVEL else LEVEL_TRITS
    return field(va, lo, width)


class Mmu:
    def __init__(self, bus: Bus) -> None:
        self.bus = bus
        self.tlb: dict[tuple[int, int], Leaf] = {}  # (asid, virtual page)
        self.global_tlb: dict[int, Leaf] = {}  # virtual page

    def translate(
        self, va: int, access: Access, ring: int, sua: bool, ptbr: Ptbr
    ) -> int:
        vpn = page_number(va)
        leaf = self.global_tlb.get(vpn) or self.tlb.get((ptbr.asid, vpn))
        if leaf is None:
            leaf = self._walk(va, access, ptbr)
            self._check(leaf.pte, va, access, ring, sua)
            if field(leaf.pte, G, 1) == 1:
                self.global_tlb[vpn] = leaf
            else:
                self.tlb[(ptbr.asid, vpn)] = leaf
        else:
            self._check(leaf.pte, va, access, ring, sua)
        return leaf.page_centre + low(va, PAGE_TRITS)

    def fence(self, va: int | None, asid: int | None) -> None:
        """fence.vm: va and asid are None when their register field is r0."""
        vpn = None if va is None else page_number(va)
        if asid is None:
            self.tlb = {k: v for k, v in self.tlb.items() if vpn not in (None, k[1])}
            if vpn is None:
                self.global_tlb.clear()
            else:
                self.global_tlb.pop(vpn, None)
            return
        # Naming an address space leaves global entries alone.
        self.tlb = {
            k: v
            for k, v in self.tlb.items()
            if not (k[0] == asid and vpn in (None, k[1]))
        }

    def _read_entry(self, addr: int, va: int) -> int:
        try:
            return self.bus.read(addr, 3, "walk")
        except BusFault:
            raise Trap(isa.ACCESS_FAULT, va) from None

    def _walk(self, va: int, access: Access, ptbr: Ptbr) -> Leaf:
        fault = Trap(FAULT_CAUSE[access], va)
        table = ptbr.root * pow3(PAGE_TRITS)
        for level in (2, 1, 0):
            pte = self._read_entry(table + 3 * index(va, level), va)
            kind = field(pte, V, 1)
            if kind == 0:
                raise fault
            if kind == -1:
                # A pointer: carries no permissions, and the leaf level has
                # nothing below it to point to.
                if level == 0 or field(pte, R, 7) != 0:
                    raise fault
                table = page_number(pte) * pow3(PAGE_TRITS)
                continue
            return self._leaf(pte, va, level, fault)
        raise AssertionError("unreachable: level 0 always returns or faults")

    @staticmethod
    def _leaf(pte: int, va: int, level: int, fault: Trap) -> Leaf:
        if any(field(pte, t, 1) == -1 for t in range(R, S)):
            raise fault  # '-' in a flag trit is reserved
        span = LEVEL_TRITS * level  # trits of virtual page number the leaf covers
        ppn = page_number(pte)
        if low(ppn, span) != 0:
            raise fault  # a superpage must be aligned to its own size
        # The base page this va falls in, within a possibly larger leaf.
        within = split_high(low(va, PAGE_TRITS + span), PAGE_TRITS)
        return Leaf(pte, (ppn + within) * pow3(PAGE_TRITS))

    @staticmethod
    def _check(pte: int, va: int, access: Access, ring: int, sua: bool) -> None:
        def has(t: int) -> bool:
            return field(pte, t, 1) == 1

        fault = Trap(FAULT_CAUSE[access], va)
        if not all(has(t) for t in NEEDS[access]):
            raise fault
        if ring < 0 and not has(U):
            raise fault
        if ring >= 0 and has(U) and (access is Access.FETCH or not sua):
            raise fault  # the kernel never runs user code, and reads it only with SUA
        if not has(A):
            raise fault
        if access in (Access.STORE, Access.AMO) and not has(D):
            raise fault
