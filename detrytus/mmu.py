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
from detrytus.trits import field, low, pow3, split_high, to_trits

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
    """A cached translation: the leaf PTE, its flag trits (index by V, R,
    W, ...) and the physical centre of the base page the virtual page maps
    to."""

    pte: int
    page_centre: int
    flags: tuple[int, ...]


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
        # Translations already checked for one kind of access from one
        # ring: (asid, virtual page, access, ring, sua) -> page centre. It
        # is derived from the TLB and cleared with it by fence(), so it
        # never outlives a TLB entry; it only skips repeating the check.
        self._checked: dict[tuple[int, int, Access, int, bool], int] = {}

    def translate(
        self, va: int, access: Access, ring: int, sua: bool, ptbr: Ptbr
    ) -> int:
        vpn = split_high(va, PAGE_TRITS)
        key = (ptbr.asid, vpn, access, ring, sua)
        centre = self._checked.get(key)
        if centre is not None:
            return centre + low(va, PAGE_TRITS)
        leaf = self.global_tlb.get(vpn) or self.tlb.get((ptbr.asid, vpn))
        if leaf is None:
            leaf = self._walk(va, access, ptbr)
            self._check(leaf.flags, va, access, ring, sua)
            if leaf.flags[G] == 1:
                self.global_tlb[vpn] = leaf
            else:
                self.tlb[(ptbr.asid, vpn)] = leaf
        else:
            self._check(leaf.flags, va, access, ring, sua)
        self._checked[key] = leaf.page_centre
        return leaf.page_centre + low(va, PAGE_TRITS)

    def probe(self, va: int, ptbr: Ptbr) -> int | None:
        """Translate without checking permissions or touching the TLB."""
        try:
            leaf = self._walk(va, Access.LOAD, ptbr)
        except Trap:
            return None
        return leaf.page_centre + low(va, PAGE_TRITS)

    def fence(self, va: int | None, asid: int | None) -> None:
        """fence.vm: va and asid are None when their register field is r0."""
        self._checked.clear()  # cheaper to rebuild than to filter
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
        flags = tuple(to_trits(pte, PAGE_TRITS))
        if any(flags[t] == -1 for t in range(R, S)):
            raise fault  # '-' in a flag trit is reserved
        span = LEVEL_TRITS * level  # trits of virtual page number the leaf covers
        ppn = page_number(pte)
        if low(ppn, span) != 0:
            raise fault  # a superpage must be aligned to its own size
        # The base page this va falls in, within a possibly larger leaf.
        within = split_high(low(va, PAGE_TRITS + span), PAGE_TRITS)
        return Leaf(pte, (ppn + within) * pow3(PAGE_TRITS), flags)

    @staticmethod
    def _check(
        flags: tuple[int, ...], va: int, access: Access, ring: int, sua: bool
    ) -> None:
        user = flags[U] == 1
        allowed = (
            all(flags[t] == 1 for t in NEEDS[access])
            and (ring >= 0 or user)
            # the kernel never runs user code, and reads it only with SUA
            and not (ring >= 0 and user and (access is Access.FETCH or not sua))
            and flags[A] == 1
            and not (access in (Access.STORE, Access.AMO) and flags[D] != 1)
        )
        if not allowed:
            raise Trap(FAULT_CAUSE[access], va)
