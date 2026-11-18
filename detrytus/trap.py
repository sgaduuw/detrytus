"""Exceptions that carry traps through the emulator."""


class Trap(Exception):
    """Raised by an instruction to enter a trap with this cause and tval."""

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
