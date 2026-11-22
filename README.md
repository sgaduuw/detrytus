# detrytus

A balanced-ternary computer: an instruction set, an assembler, an
emulator and a debugger. If you came here because of the Soviet
**Setun** (1958) or **Setun-70** (1970), the ternary computers built at
Moscow State University, this is a project in their line. It asks what a
ternary machine would look like if it had kept evolving alongside binary
CPUs for another fifty years.

It is **not** a Setun emulator or replica. It cannot run Setun
programs, and its word size, instruction set and design are its own.
What it shares with the Setun is the number system, balanced ternary,
and the conviction that a machine built on it can be simpler than a
binary one.

The name is detritus with a tryte in it.

## Setun and detrytus

Balanced ternary writes numbers with trits of −1, 0 and +1. The Setun
used it because it removes a lot of special cases: there is no sign
bit, negating a number inverts each trit, truncating rounds to the
nearest value, and a comparison naturally has three outcomes.
detrytus keeps all of that. Everything else is a modern design.

| | Setun (1958) and Setun-70 (1970) | detrytus |
|---|---|---|
| Number system | balanced ternary | balanced ternary |
| Word | 18 trits (Setun), with 9-trit short words | 27 trits; a tryte is 9 trits |
| Architecture | accumulator machine (Setun); two-stack machine (Setun-70) | RISC load/store, 27 registers |
| Hardware | ferrite cores and diodes, about 50 Setuns built | a specification and a software emulator; no hardware |
| Operating system support | none in the modern sense | three privilege rings, traps, paged virtual memory, a timer, interrupts, multiple harts with atomics |

So detrytus is ternary in the way the Setun was, and modern in the way
a RISC-V or ARM core is. Where ternary lets a feature be simpler than
its binary equivalent, the specification takes the simpler form and
says why ([`docs/isa.md`](docs/isa.md) section 2).

## The shape of it

- Trits take the values −1, 0, +1. A tryte is 9 trits, a word is 27.
- 27 registers indexed −13..+13, with `r0` hardwired to zero.
- 27-trit instructions. A word can instead be a bundle of two 11-trit
  compressed instructions, for density.
- A compare yields a trit, and `br3` branches three ways on it: the
  instruction the Setun's design points at.
- One integer type: no sign bit, no unsigned variants, and `not` and
  `neg` are the same instruction.
- One `tlut` instruction expresses any of the 19,683 two-input trit
  functions.
- Pages, words and tables are centred on their addresses, so an address
  splits into page number and offset just by cutting trits.
- The sign of a physical page number picks RAM, boot firmware or
  devices; the sign of a trap cause picks interrupt or exception.
- Position-independent code, with a global pointer (`gp`) for short
  access to a program's data.

## Status

The instruction set is a draft, version 0.11, specified in
[`docs/isa.md`](docs/isa.md). It will change until the emulator and the
operating system written for it have settled.

The `detrytus` command, written in Rust, assembles, runs and debugs
programs for the integer core. Implemented:

- every integer, tritwise, memory, atomic, control and system
  instruction
- traps, delegation across three rings, the timer, single step and
  paged virtual memory
- up to 729 harts, with a mailbox between them, `wrs` waits and
  hot-plug
- an interrupt controller, a console and a block device backed by a
  file on the host
- compressed bundles, which the assembler forms by itself under
  `.compress`, and gp-relative addressing under `.gp`

Floating point and debug triggers are specified but not implemented
yet.

The machine has an operating system being written for it,
[trylobite](https://github.com/sgaduuw/trylobite): a microkernel with
drivers and the filesystem as server processes, a shell with pipes and
redirection, written in detrytus assembly. It is the quickest way to
see the machine do something real. Running Linux is not a goal: C and the Linux kernel
assume binary integers and 8-bit bytes throughout
([`CONTEXT.md`](CONTEXT.md) explains why).

## Try it

Requires a Rust toolchain, 1.88 or later.

```bash
cargo install --git https://github.com/sgaduuw/detrytus detrytus-cli
```

or, from a checkout:

```bash
cargo install --path rust/cli
```

Then:

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

`--input FILE` gives `run` or `debug` text for the program to read from
the console, `-` meaning stdin:

```bash
echo "hello" | detrytus run --input - examples/echo.s
```

Started at a terminal without `--input`, `run` has a **live console**
instead: what you type reaches the program as you type it, and the run
goes on, with no step limit, until the program powers off. The terminal
is switched to pass every key through (no line editing or echo of its
own, so backspace and Ctrl-C reach the program, which echoes what it
reads), and switched back when the run ends. Ctrl-D closes the
console's input; Ctrl-] stops the machine (exit status 130). While
every hart waits for the clock, it moves at about a million ticks a
second rather than skipping ahead, so a program that sleeps does so in
real time (docs/platform.md, Time). `--live`
asks for this from a pipe too, passing bytes on as they arrive and
closing the console when the pipe ends.

`--harts N` starts the machine with N harts (1 to 729), and
`--plug TIME:HART` / `--unplug TIME:HART` add or remove one while it
runs:

```bash
detrytus run --harts 3 examples/harts.s
```

`--disk FILE` attaches a block device whose blocks live in FILE, two
bytes per tryte; `--disk-blocks N` sets its size and creates the file if
it is missing. What a program writes stays in the file for the next run.

`--mix` counts instructions by mnemonic and how many of each a
compressed form (docs/isa.md 6.1) could stand for: `run --mix` counts
what executed, `asm --mix` what a source file holds, with how many words
pairing compressible neighbours into bundles would save. It is how the
compressed table is judged against real code.

Running `detrytus` with no arguments lists every option.

## Documentation

- [`docs/isa.md`](docs/isa.md): the instruction set, the source of
  truth for what the machine does.
- [`docs/assembler.md`](docs/assembler.md): source syntax, directives
  (`.compress`, `.gp`, `.assert` and the rest) and pseudo-instructions.
- [`docs/platform.md`](docs/platform.md): the emulated memory map and
  devices.
- [`docs/debugger.md`](docs/debugger.md): the debugger's commands.
- [`CONTEXT.md`](CONTEXT.md): the decisions behind the design and why
  they were made.
- [`docs/rust-plan.md`](docs/rust-plan.md): how the implementation
  moved from Python to Rust. The first implementation was Python; the
  Rust tools reproduced its output exactly before it was removed.
- [`examples/`](examples/): runnable programs.
- [`conformance/`](conformance/README.md): the conformance suite,
  machine behaviour as data that any implementation runs.
- [trylobite](https://github.com/sgaduuw/trylobite): the operating
  system written for the machine.

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

## Development

[`AGENTS.md`](AGENTS.md) describes how changes are made. These checks
all run in CI:

```bash
cargo test
```

```bash
cargo clippy --all-targets -- -D warnings
```

```bash
cargo fmt --check
```

## License

MIT, see [`LICENSE`](LICENSE).
