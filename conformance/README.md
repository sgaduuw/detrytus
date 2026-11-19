# The conformance suite

Machine behaviour, stored as data: each case is an assembly program and
what running it must produce. Any implementation of detrytus runs the
same cases, so they outlive the Python emulator they were first written
for (see [`docs/rust-plan.md`](../docs/rust-plan.md)).

```bash
uv run pytest tests/test_conformance.py
```

The Python runner is [`detrytus/conformance.py`](../detrytus/conformance.py).

## Layout

| Directory | Covers |
|-----------|--------|
| `cpu/` | integer and tritwise instructions, branches, memory, atomics |
| `traps/` | traps, delegation, interrupts, the timer, single step |
| `paging/` | translation, permission faults, superpages, the TLB |
| `examples/` | the programs in `examples/`, run as shipped |

`paging/` also holds shared assembly: `tables.s` (constants and base
page tables), `boot.s` (firmware that enables paging and drops to a
ring), and `user.s` and `kernel.s`, which put them together for a
program that runs in ring −1 or ring 0. Page tables are built
statically with `.org` and `.word`, so a case's whole machine state is
in its source.

## Format

A file holds one or more cases, and optionally defaults for them:

```toml
[defaults]
postlude = '''
        li    t6, -39366
        sw    a0, 0(t6)
'''

[[case]]
name = "add"
program = "li a1, 7\nli a2, -3\nadd a0, a1, a2"
expect.exit_code = 4
expect.registers = { a1 = 7, a2 = -3 }
```

### Case keys

| Key | Meaning |
|-----|---------|
| `name` | required; unique in the file, no `/` |
| `program` | assembly source, inline |
| `source` | a file to assemble, relative to the case file |
| `sources` | several files (sources or `.dti` images) loaded together |
| `prelude`, `postlude` | text put before and after `program` |
| `ram_pages` | RAM size in pages; default 729 |
| `max_steps` | step limit; default 100000 |
| `input` | text queued on the console's input |

Exactly one of `program`, `source` and `sources`. Any key but `name`
may also be given in `[defaults]`, and a case's own value wins. An
inline program is assembled as if it were a file in the case file's
directory, so `.include` paths start there, and errors name the case:
`cpu/integer.toml [add]:3`. Lines count from the start of the
assembled text, prelude included.

The machine is the platform of [`docs/platform.md`](../docs/platform.md),
with its entropy source seeded with 1. The image is loaded and the hart
starts at address 0 in ring +1.

### Expectations

Under `expect`. Exactly one outcome:

| Key | The run must |
|-----|--------------|
| `exit_code` | power off with this code |
| `step_limit = true` | still be running after `max_steps` steps |
| `unhandled_cause` | stop on a trap with this cause, its bank's `tvec` being 0 |

and any of these checks on the state it stops in:

| Key | Checks |
|-----|--------|
| `console` | the console's whole output, exactly |
| `ring` | the hart's current ring |
| `registers` | `{ name = value }`, by any register name (`a0`, `r1`, `sp`) |
| `csrs` | `{ name = value }`, read with ring +1's privilege |
| `words` | `{ "address" = value }`, the word centred on a physical address |
| `trytes` | `{ "address" = value }`, a tryte at a physical address |

Values are integers in balanced range. TOML keys must be strings, so
addresses are quoted.

Unknown keys are errors, so a misspelt expectation cannot pass silently.

## Adding a case

Machine behaviour that a program can observe belongs here, not in a
Python test. Then make sure the case can fail: break the emulator code
it guards, watch the case go red, and restore it (AGENTS.md, Tests).
Tests of the Python tools themselves (assembler, debugger, image
format, spec sync) stay in `tests/`.
