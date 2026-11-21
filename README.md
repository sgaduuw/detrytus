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
The `detrytus` command, written in Rust, assembles, runs and debugs
programs for its integer core: every integer, tritwise, memory, atomic,
control and system instruction, plus traps, delegation, interrupts, the
timer, single step, paged virtual memory and compressed bundles
(running them; the assembler does not write them yet). Floating point
and debug triggers are specified but not implemented yet.

The first implementation was Python. The Rust tools replaced it
([`docs/rust-plan.md`](docs/rust-plan.md)), producing exactly its output
before it was removed, and run trylobite about 16 times faster.

## Usage

Requires a Rust toolchain, 1.88 or later.

```bash
cargo install --path rust/cli
```

or, without a checkout:

```bash
cargo install --git https://github.com/sgaduuw/detrytus detrytus-cli
```

```bash
detrytus run examples/hello.s
```

```bash
detrytus asm examples/bsearch.s
```

```bash
detrytus debug examples/bsearch.s
```

`run` assembles a program, loads it at address 0 and runs it until it
writes to the power-off device. It takes several files, sources or
`.dti` images written by `asm -o`, and loads them together. `debug` stops before the first
instruction for stepping, breakpoints, watchpoints and inspection. `--stats` prints
the step count and `--trace` every instruction. `asm` prints a listing with each word's trits
and its disassembly.

`--mix` counts instructions by mnemonic and how many of each a
compressed form (docs/isa.md 6.1) could stand for: `run --mix` counts
what executed, `asm --mix` what a source file holds, with how many words
pairing compressible neighbours into bundles would save. It is how the
compressed table is judged against real code.

`--input FILE` gives `run` or `debug` text for the program to read from
the console, `-` meaning stdin:

```bash
echo "hello" | detrytus run --input - examples/echo.s
```

`--disk FILE` attaches a block device whose blocks live in FILE, two
bytes per tryte; `--disk-blocks N` sets its size and creates the file if
it is missing. What a program writes stays in the file for the next run.

- [`docs/assembler.md`](docs/assembler.md): source syntax, directives
  and pseudo-instructions.
- [`docs/platform.md`](docs/platform.md): the emulated memory map and
  devices.
- [`docs/debugger.md`](docs/debugger.md): the debugger's commands.
- [`docs/rust-plan.md`](docs/rust-plan.md): how the implementation
  moved from Python to Rust.
- [`examples/`](examples/): runnable programs.
- [`conformance/`](conformance/README.md): the conformance suite,
  machine behaviour as data that any implementation runs.

Development checks, all run in CI:

```bash
cargo test
```

```bash
cargo clippy --all-targets -- -D warnings
```

```bash
cargo fmt --check
```

## Layout

| Path | Job |
|------|-----|
| `rust/core/` | the machine: trits, the instruction set, the hart, privilege, paging, the bus and devices, images; no dependencies |
| `rust/asm/` | the assembler and disassembler |
| `rust/cli/` | the `detrytus` command: `asm`, `run` and `debug` |
| `rust/conformance/` | runs the conformance suite on the assembler and core |
| `conformance/` | the conformance suite, as data |
| `docs/` | the specification and the tools' manuals |
| `examples/` | runnable programs |

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
