"""Physical memory: the boot page, RAM and memory-mapped devices.

The region an address belongs to is decided by the sign of its physical
page number (docs/isa.md 5.3), so routing an access is one comparison,
with no attribute table.
"""

from typing import Protocol

from detrytus.trits import TRYTE_MOD, TRYTE_TRITS, low, split_high

PAGE_TRITS = 9


class Device(Protocol):
    def read(self, addr: int, width: int, /) -> int: ...
    def write(self, addr: int, width: int, value: int, /) -> None: ...


class BusFault(Exception):
    """An access to an address with nothing behind it, or a forbidden one."""

    def __init__(self, addr: int, kind: str) -> None:
        super().__init__(f"{kind} access fault at {addr}")
        self.addr = addr
        self.kind = kind


def page_of(addr: int) -> int:
    return split_high(addr, PAGE_TRITS)


class Bus:
    """Tryte-addressed physical memory.

    Memory is sparse, a dict from address to tryte, so a large RAM costs
    nothing until it is touched. ``ram_pages`` bounds the positive page
    numbers that count as RAM.
    """

    def __init__(self, ram_pages: int = 729) -> None:
        self.ram_pages = ram_pages
        self.mem: dict[int, int] = {}
        self.devices: dict[int, Device] = {}

    def attach(self, page: int, device: Device) -> None:
        if page >= 0:
            raise ValueError("devices live at negative page numbers")
        self.devices[page] = device

    def poke(self, addr: int, tryte: int) -> None:
        """Write memory without access checks: for loaders, not the CPU."""
        self.mem[addr] = low(tryte, TRYTE_TRITS)

    def _check(self, addr: int, kind: str) -> Device | None:
        page = page_of(addr)
        if page == 0:
            if kind == "store":
                raise BusFault(addr, kind)
            return None
        if page > 0:
            if page > self.ram_pages:
                raise BusFault(addr, kind)
            return None
        device = self.devices.get(page)
        if device is None or kind == "fetch":
            raise BusFault(addr, kind)
        return device

    def read(self, addr: int, width: int, kind: str = "load") -> int:
        """Read a tryte (width 1) or a word (width 3) centred on addr.

        A word's alignment is the caller's to check; all three of its
        trytes then fall in one page, so one region check covers them.
        """
        device = self._check(addr, kind)
        if device is not None:
            return device.read(addr, width)
        m = self.mem
        if width == 1:
            return m.get(addr, 0)
        return (
            m.get(addr - 1, 0)
            + m.get(addr, 0) * TRYTE_MOD
            + m.get(addr + 1, 0) * TRYTE_MOD**2
        )

    def write(self, addr: int, width: int, value: int) -> None:
        device = self._check(addr, "store")
        if device is not None:
            device.write(addr, width, value)
            return
        if width == 1:
            self.mem[addr] = low(value, TRYTE_TRITS)
            return
        lo = low(value, TRYTE_TRITS)
        rest = (value - lo) // TRYTE_MOD
        mid = low(rest, TRYTE_TRITS)
        self.mem[addr - 1] = lo
        self.mem[addr] = mid
        self.mem[addr + 1] = (rest - mid) // TRYTE_MOD
