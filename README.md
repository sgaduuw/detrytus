# detrytus

A balanced-ternary CPU architecture, picking up where the Soviet Setun
(1958) and Setun-70 (1970) left off and adding what a modern operating
system expects from a core: a RISC load/store design, position-independent
code, three privilege rings with trap delegation, paged virtual memory,
a timer, acquire/release atomics, floating point and cache control.

The target is an operating system of its own (a microkernel, as a
separate project). Running Linux is not a goal: C and the kernel assume
binary integers and 8-bit bytes throughout.

The name is detritus with a tryte in it.

## Status

The instruction set is a draft, specified in [`docs/isa.md`](docs/isa.md).
A Python emulator and assembler implement its integer core: every
integer, tritwise, memory, atomic, control and system instruction, plus
traps, delegation, interrupts, the timer, single step and paged virtual
memory. Floating point, compressed bundles and debug triggers are
specified but not implemented yet. A Rust implementation may follow once the design
settles.

## Usage

Requires [uv](https://docs.astral.sh/uv/).

```bash
uv sync
```

```bash
uv run detrytus run examples/hello.s
```

```bash
uv run detrytus asm examples/bsearch.s
```

```bash
uv run detrytus debug examples/bsearch.s
```

`run` assembles a program, loads it at address 0 and runs it until it
writes to the power-off device. `debug` stops before the first
instruction for stepping, breakpoints, watchpoints and inspection. `--trace` prints every instruction and
`--stats` the step count. `asm` prints a listing with each word's trits
and its disassembly.

- [`docs/assembler.md`](docs/assembler.md): source syntax, directives
  and pseudo-instructions.
- [`docs/platform.md`](docs/platform.md): the emulated memory map and
  devices.
- [`docs/debugger.md`](docs/debugger.md): the debugger's commands.
- [`examples/`](examples/): runnable programs.

Development checks, all run in CI:

```bash
uv run pytest
```

```bash
uv run mypy
```

```bash
uv run ruff check detrytus tests
```

```bash
uv run ruff format --check detrytus tests
```

## Layout

| Module | Job |
|--------|-----|
| `trits.py` | balanced-ternary arithmetic on Python ints |
| `isa.py` | the instruction set as data: opcodes, formats, CSRs, encode and decode |
| `alu.py` | integer and tritwise operations as pure functions |
| `execute.py` | what each instruction does: one handler per mnemonic, in one table |
| `trap.py` | the exceptions that carry traps |
| `mmu.py` | paged virtual memory: the walk, permission checks, the TLB |
| `cpu.py` | one hart: fetch loop, traps, interrupts, CSRs |
| `bus.py` | physical memory and device routing |
| `devices.py` | console and power control |
| `machine.py` | the platform wired together |
| `expr.py` | assembler expressions |
| `asm/` | the assembler: `source` parsing, `operands`, `pseudo`-instructions, `core` passes |
| `disasm.py` | the disassembler |
| `debug/` | the debugger: `engine` runs and inspects, `shell` is the front end |
| `cli.py` | the `detrytus` command |

## The shape of it

- Trits take the values −1, 0, +1. A tryte is 9 trits, a word is 27.
- 27 registers indexed −13..+13, with `r0` hardwired to zero.
- Fixed 27-trit instructions.
- A compare yields a trit, and `br3` branches three ways on it.
- One integer type: no sign bit, no unsigned variants, and `not` and
  `neg` are the same instruction.
- One `tlut` instruction expresses any of the 19,683 two-input trit
  functions.
- Pages, words and tables are centred on their addresses, so an address
  splits into page number and offset just by cutting trits.
- The sign of a physical page number picks RAM, boot firmware or
  devices; the sign of a trap cause picks interrupt or exception.

## License

MIT, see [`LICENSE`](LICENSE).
