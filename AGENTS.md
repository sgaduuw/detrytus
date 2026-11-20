# AGENTS.md

Instructions for AI coding agents working on detrytus. Read this before
your first change. Its companion, [`CONTEXT.md`](CONTEXT.md), records the
decisions behind the design and why they were made; read the relevant
entries before changing anything they cover, and add an entry when you
make a decision of the same weight.

## What detrytus is

A balanced-ternary CPU architecture in the line of the Soviet Setun
(1958) and Setun-70 (1970), reshaped around what a modern operating
system expects from a core. The repository holds three things:

- **The ISA specification**, [`docs/isa.md`](docs/isa.md). It is the
  source of truth for what the machine does.
- **A Python emulator and assembler** that implement the spec, and the
  reference while the implementation moves to Rust.
- **A Rust core** (`rust/core/`), the machine without the tools, built
  up phase by phase per [`docs/rust-plan.md`](docs/rust-plan.md).
- **Examples and tests** that exercise both.

The target is an operating system of its own, a microkernel in a
separate project. Running Linux or other C-era binary software is not a
goal (CONTEXT.md explains why).

## Tooling

[uv](https://docs.astral.sh/uv/) for Python, pytest for tests, mypy
(strict) for types, ruff for lint and formatting.

```sh
uv sync
uv run pytest
uv run mypy
uv run ruff check detrytus tests
uv run ruff format --check detrytus tests
uv run detrytus run examples/hello.s
```

For the Rust core, a toolchain of 1.88 or later:

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

CI runs all seven checks on every pull request, and a third job builds
the optional Rust backend (`uv pip install ./rust/py`) and runs the
conformance and differential tests on it. Run them yourself before
pushing, each as its own command, and judge each by its exit status. A
pipe (`ruff check ... | tail`) replaces the tool's status with the
pipe's and hides a failure.

When formatting, scope ruff to the files you touched so unrelated drift
stays out of your diff.

## Where code goes

| Path | Job |
|------|-----|
| `docs/isa.md` | the specification |
| `docs/assembler.md` | assembler syntax |
| `docs/platform.md` | the emulated memory map and devices |
| `docs/debugger.md` | the debugger's commands |
| `detrytus/trits.py` | balanced-ternary arithmetic on Python ints |
| `detrytus/isa.py` | the instruction set as data: opcodes, formats, CSRs, encode and decode |
| `detrytus/alu.py` | integer and tritwise operations as pure functions |
| `detrytus/execute.py` | what each instruction does: one handler per mnemonic |
| `detrytus/trap.py` | the exceptions that carry traps |
| `detrytus/mmu.py` | paged virtual memory: the walk, permission checks, the TLB |
| `detrytus/cpu.py` | one hart: fetch loop, traps, interrupts, CSRs |
| `detrytus/bus.py`, `devices.py` | physical memory, device routing, devices |
| `detrytus/machine.py` | the platform wired together |
| `detrytus/expr.py` | assembler expressions |
| `detrytus/asm/` | the assembler: `source`, `operands`, `pseudo`, `core`, `errors` |
| `detrytus/image.py` | the `.dti` image format and merging images |
| `detrytus/disasm.py` | the disassembler |
| `detrytus/debug/` | the debugger: `engine` (run control, inspection), `shell` (commands) |
| `detrytus/conformance.py` | the conformance suite's runner |
| `detrytus/cli.py` | the `detrytus` command |
| `conformance/` | the conformance suite: machine and assembler behaviour as data |
| `rust/core/src/` | the Rust core: `trits`, `isa`, `alu`, `bus`, `image`, `hart`, `privileged`, `mmu`, `entropy`, `machine` |
| `rust/core/examples/run.rs` | runs `.dti` images on the Rust core, for trying and timing it |
| `rust/py/` | the optional `detrytus-rs` package: Python bindings for the Rust core |
| `rust/asm/src/` | the Rust assembler: `expr`, `source`, `operands`, `pseudo`, `assembler`, `disasm` |
| `rust/cli/` | the Rust `detrytus` command: `asm`, `run` and `debug` (`program.rs` loads files, `debug/` is the engine and shell) |
| `rust/conformance/` | the Rust conformance runner (tests only) |
| `detrytus/rust.py` | wraps those bindings in the Python core's interface |
| `rust/core/tests/` | the Rust spec sync test |
| `examples/` | runnable assembly programs |

One concern per file. If a module's job needs an "and" to describe, it
wants splitting, as the assembler was into a package.

## The spec and the code move together

`docs/isa.md` and `detrytus/isa.py` describe the same instruction set.
`tests/test_spec_sync.py` parses the spec's opcode, truth-table and CSR
tables and fails if the code disagrees. So:

- A change to an encoding, opcode, CSR number or table value changes
  **both** files in the same commit.
- Bump the version line at the top of `docs/isa.md` when the spec's
  meaning changes, and record why in CONTEXT.md.
- Spec decisions belong to the project owner. Propose them; do not make
  them silently in code.

### Adding an instruction

1. Specify it in `docs/isa.md`: opcode, format, semantics.
2. Add its row to `_TABLE` in `isa.py`, with its operand syntax letters.
3. Add its handler to `HANDLERS` in `execute.py`. A missing or extra
   handler stops the package from importing, by design.
4. If it uses a new operand letter, add a setter to
   `asm/operands.py` and a renderer to `disasm.py`.
5. Test its semantics, its encoding round trip, and its illegal forms.

## Python and Rust move together

While both implementations exist, a change to machine or assembler
behaviour changes both, and a conformance case pins it down. Python is
the reference: when they disagree, Python is right until the spec says
otherwise. The Rust assembler reproduces the Python one's output
exactly, image bytes and error messages included, so a change to a
message changes both.

The Rust core has no dependencies; its tests may have some. It stops
with `Stop::Unsupported` for anything it does not implement yet, as the
Python core raises `NotImplementedError`, and never approximates.

## Conventions

- **Values are Python ints in balanced range**, not trit lists. Use the
  helpers in `trits.py` (`low`, `field`, `split_high`, `set_trit`,
  `wrap`, `pow3`) rather than re-deriving the arithmetic.
- **Memory layout is centred**: a word at address `a` covers `a-1`,
  `a`, `a+1`. So are pages and tables. Do not "fix" this to start-based;
  CONTEXT.md explains it.
- **Absolute imports only** (`from detrytus import isa`), enforced by
  ruff. Import `isa` by name, never under a short alias.
- **Strict types.** Every function is annotated and `uv run mypy`
  passes. For a power of three with a variable exponent use `pow3(n)`,
  since `3**n` types as `Any`.
- **Keep functions small.** Ruff enforces complexity 10. When a function
  grows an `if`/`elif` chain over kinds of thing, make it a table of
  small functions, as `execute.py`, `asm/operands.py` and `disasm.py`
  do.
- **Name constants** that come from the spec instead of writing raw
  numbers (`TRYTE_MOD`, not `19683`).
- **Unimplemented features fail loudly.** If the emulator meets
  something the spec defines but the code does not implement yet, raise
  `NotImplementedError` with a clear message. Never approximate.
- **Comments explain why**, not what.
- **No em-dashes or double hyphens in prose**: comments, docstrings,
  docs, commit messages. Use commas, colons or parentheses.
- **Keep it simple.** No dependencies without a strong reason; the
  package has none at runtime. Add indirection only when a second user
  of it exists.

## Tests

When you change the ISA tables, the decoder, the emulator's semantics,
the assembler's layout or encoding, add a test rather than checking by
hand. Behaviour a program can observe (results, traps, paging, the
console) is tested by a case in [`conformance/`](conformance/README.md),
not in Python, so the Rust core will be held to it too. Tests of the
Python tools themselves stay in `tests/`. Then make sure the test can
fail:

1. **Watch it fail.** Break the code your test guards and confirm the
   test goes red, then restore the code. A test never seen to fail is
   not a guard.
2. **Turn off bytecode caching while doing that**
   (`PYTHONDONTWRITEBYTECODE=1`, and remove `__pycache__`). An edit
   that keeps a file's size, such as `35` to `38`, restored within the
   same second, can leave Python running the stale bytecode and make a
   later run lie.
3. **Assert the precondition.** Check that the fixture builds the
   situation the test exists for.
4. **Read registers and CSRs through their names** (`hart.reg("a1")`,
   `hart.read_csr(isa.CSRS["mtcause"])`), not by indexing internal
   lists.

Tests that need the Rust backend use `needs_rust` from
`tests/backends.py`: they skip without the `detrytus-rs` package, and
fail instead when `DETRYTUS_REQUIRE_RUST` is set, as in CI.
`tests/test_differential.py` runs both cores step by step and compares
their whole state; a change to either core that it flags is a
disagreement to resolve, with the Python core as the reference.
`tests/test_cli_parity.py` runs the Rust and Python `detrytus asm`,
`run` and `debug` on every conformance program, the examples, a corpus
of broken source and scripted debugger sessions, and wants identical
stdout, stderr, exit status and image bytes; it needs `cargo build`
first, and a change to either command's output changes both.

## Git and pull requests

- `main` holds merged, CI-green work. Branch from it for each change
  and open a pull request; CI must be green before merge.
- Agents do not merge. The owner merges.
- Keep a pull request to its title. Revert unrelated drift rather than
  bundling it.
- Commit messages: a short imperative subject, then what changed and
  why.
- Start each new piece of work from a fresh branch off the latest
  `main`. Merged branches may be deleted automatically.

## When unsure

Ask rather than guess, especially about the spec, the direction of the
project, or anything CONTEXT.md marks as an open question.
