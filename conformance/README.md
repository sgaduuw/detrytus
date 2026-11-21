# The conformance suite

Machine and assembler behaviour, stored as data: each case is an
assembly program and what assembling and running it must produce. Any
implementation of detrytus could run the same cases: they were first
run on the Python tools, then held the Rust ones to the same results
before the Python ones were removed (see
[`docs/rust-plan.md`](../docs/rust-plan.md)).

```bash
cargo test -p detrytus-conformance
```

The runner, [`rust/conformance/tests/conformance.rs`](../rust/conformance/tests/conformance.rs),
checks each file's format, assembles each case and runs it on the core.

## Layout

| Directory | Covers |
|-----------|--------|
| `cpu/` | integer and tritwise instructions, branches, memory, atomics |
| `traps/` | traps, delegation, interrupts, the timer, single step |
| `paging/` | translation, permission faults, superpages, the TLB |
| `harts/` | several harts: starting and stopping, mailboxes, group sends, `wfi`, the shared clock, hot-plug |
| `devices/` | the interrupt controller and console input states |
| `asm/` | the assembler: layout, local labels, sections, and its errors |
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
| `console_open` | `true` leaves the console open once `input` is read, as the debugger does: reads give `CONSOLE_EMPTY` (−1) rather than `CONSOLE_CLOSED` (−2); default false, as `run` |
| `harts` | harts present at reset, numbered outward from 0; default 1 |
| `hotplug` | a list of `{ time = T, add = H }` or `{ time = T, remove = H }` |

Exactly one of `program`, `source` and `sources`. Any key but `name`
may also be given in `[defaults]`, and a case's own value wins. An
inline program is assembled as if it were a file in the case file's
directory, so `.include` paths start there, and errors name the case:
`cpu/integer.toml [add]:3`. Lines count from the start of the
assembled text, prelude included.

The machine is the platform of [`docs/platform.md`](../docs/platform.md),
with its entropy source seeded with 1. The image is loaded and hart 0
starts at address 0 in ring +1.

### Expectations

Under `expect`. Exactly one outcome:

| Key | The run must |
|-----|--------------|
| `exit_code` | power off with this code |
| `step_limit = true` | still be running after `max_steps` steps |
| `unhandled_cause` | stop on a trap with this cause, its bank's `tvec` being 0, on hart `unhandled_hart` (default 0) |
| `asm_error` | fail to assemble, with exactly this message |

`asm_error` is the message alone, without the file, line and source line
an assembler prints around it; `asm_error_line` may give the line it
must be reported at. A case that expects an assembly error takes no
other keys under `expect`.


and, for a run, any of these checks on the state it stops in:

| Key | Checks |
|-----|--------|
| `console` | the console's whole output, exactly |
| `ring` | hart 0's current ring |
| `registers` | `{ name = value }`, hart 0's, by any register name (`a0`, `r1`, `sp`) |
| `csrs` | `{ name = value }`, hart 0's, read with ring +1's privilege |
| `words` | `{ "address" = value }`, the word centred on a physical address |
| `trytes` | `{ "address" = value }`, a tryte at a physical address |

Values are integers in balanced range. TOML keys must be strings, so
addresses are quoted.

Unknown keys are errors, so a misspelt expectation cannot pass silently.

## Adding a case

Behaviour a program can observe, and how the assembler lays a program
out or rejects it, belongs here, not in a Rust test. Then make sure the
case can fail: break the code it guards, watch the case go red, and
restore it (AGENTS.md, Tests). What only the command's interface shows
(listing text, the debugger, include files on disk, messages) is held to
the transcripts in `rust/cli/tests/golden/`.
