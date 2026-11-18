"""Shared test helpers: assemble a snippet, run it, read the outcome."""

from detrytus.asm import assemble
from detrytus.machine import Machine

# Power off with a0 as the exit code.
HALT = """
        li    t6, -39366
        sw    a0, 0(t6)
"""


def run(source: str, max_steps: int = 100_000) -> tuple[Machine, int]:
    """Run source to power-off; return the machine and the exit code."""
    m = Machine(seed=1)
    m.load(assemble(source))
    result = m.run(max_steps)
    return m, result.exit_code


def result_of(body: str) -> int:
    """Exit code of a snippet that leaves its answer in a0."""
    return run(body + HALT)[1]
