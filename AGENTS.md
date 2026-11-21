# AGENTS.md

Instructions for AI coding agents working on detrytus. Read this before
your first change. Its companion, [`CONTEXT.md`](CONTEXT.md), records the
decisions behind the design and why they were made; read the relevant
entries before changing anything they cover, and add an entry when you
make a decision of the same weight.

## What detrytus is

A balanced-ternary CPU architecture in the line of the Soviet Setun
(1958) and Setun-70 (1970), reshaped around what a modern operating
system expects from a core. The repository holds:

- **The ISA specification**, [`docs/isa.md`](docs/isa.md). It is the
  source of truth for what the machine does.
- **The `detrytus` tools in Rust**: a core (`rust/core/`), an assembler
  (`rust/asm/`) and a command line with `asm`, `run` and `debug`
  (`rust/cli/`).
- **The conformance suite** ([`conformance/`](conformance/README.md)),
  machine and assembler behaviour as data, and **examples**.

The tools were first written in Python; the Rust ones replaced them,
output for output, and the Python ones were removed
([`docs/rust-plan.md`](docs/rust-plan.md)).

The target is an operating system of its own, a microkernel in a
separate project (trylobite). Running Linux or other C-era binary
software is not a goal (CONTEXT.md explains why).

## Tooling

A Rust toolchain, 1.88 or later.

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo run -p detrytus-cli -- run examples/hello.s
```

CI runs the three checks on every pull request. Run them yourself before
pushing, each as its own command, and judge each by its exit status. A
pipe (`cargo test | tail`) replaces the tool's status with the pipe's
and hides a failure.

## Where code goes

| Path | Job |
|------|-----|
| `docs/isa.md` | the specification |
| `docs/assembler.md` | assembler syntax |
| `docs/platform.md` | the emulated memory map and devices |
| `docs/debugger.md` | the debugger's commands |
| `rust/core/src/trits.rs` | balanced-ternary arithmetic on `i64` |
| `rust/core/src/isa.rs` | the instruction set as data: opcodes, formats, fields, CSRs, decode and encode |
| `rust/core/src/compressed.rs` | compressed bundles: slots, what a slot expands to, and which slot a full instruction fits |
| `rust/core/src/alu.rs` | integer and tritwise operations as pure functions |
| `rust/core/src/hart.rs` | one hart: the fetch loop and what each instruction does, and the events a debugger reads |
| `rust/core/src/privileged.rs` | traps, interrupts, CSRs, `tret`, `wfi` |
| `rust/core/src/mmu.rs` | paged virtual memory: the walk, permission checks, the TLB |
| `rust/core/src/bus.rs` | physical memory and the built-in devices |
| `rust/core/src/harts.rs` | the hart controller: hart states, mailboxes, group sends, hot-plug events |
| `rust/core/src/image.rs` | the `.dti` image format: reading and writing |
| `rust/core/src/entropy.rs` | the seeded entropy source |
| `rust/core/src/machine.rs` | the platform wired together, and the harts run in rounds |
| `rust/core/tests/` | the spec sync test |
| `rust/asm/src/` | the assembler: `expr`, `source`, `gp` (`.gp`), `operands`, `pseudo`, `slots` (`c.` mnemonics), `assembler`, `disasm` |
| `rust/cli/src/` | the `detrytus` command: `main.rs`, `program.rs` (loading files), `disk.rs` (`--disk`), `mix.rs` (`--mix`), `debug/` (engine and shell) |
| `rust/cli/tests/golden/` | recorded transcripts of the command's output |
| `rust/conformance/` | the conformance runner (tests only) |
| `conformance/` | the conformance suite |
| `examples/` | runnable assembly programs |

One concern per file. If a module's job needs an "and" to describe, it
wants splitting.

## The spec and the code move together

`docs/isa.md` and `rust/core/src/isa.rs` describe the same instruction
set. `rust/core/tests/spec_sync.rs` parses the spec's opcode, truth
table and CSR tables and fails if the code disagrees. So:

- A change to an encoding, opcode, CSR number or table value changes
  **both** files in the same commit.
- Bump the version line at the top of `docs/isa.md` when the spec's
  meaning changes, and record why in CONTEXT.md.
- Spec decisions belong to the project owner. Propose them; do not make
  them silently in code.

### Adding an instruction

1. Specify it in `docs/isa.md`: opcode, format, semantics.
2. Add a `Kind` variant and its row to `OPS` in `isa.rs`, with its
   operand syntax letters.
3. Give it its behaviour in `hart.rs` (or `privileged.rs`). An
   instruction with none falls through to `Stop::Unsupported`, so a
   forgotten one fails loudly.
4. If it uses a new operand letter, teach `rust/asm/src/operands.rs` to
   set it and `disasm.rs` to render it.
5. Add conformance cases for its semantics and its illegal forms, and a
   unit test for its encoding.

## Conventions

- **Values are integers in balanced range**, not trit lists: `i64` for
  words, `i128` for products and assembler expressions. Use the helpers
  in `trits.rs` (`low`, `field`, `split_high`, `set_trit`, `wrap`,
  `pow3`) rather than re-deriving the arithmetic.
- **Memory layout is centred**: a word at address `a` covers `a-1`,
  `a`, `a+1`. So are pages and tables. Do not "fix" this to start-based;
  CONTEXT.md explains it.
- **The core has no dependencies**; tests may have some (`toml`).
- **Name constants** that come from the spec instead of writing raw
  numbers (`TRYTE_MOD`, not `19683`).
- **Unimplemented features fail loudly.** If the machine meets
  something the spec defines but the code does not implement yet, stop
  with `Stop::Unsupported` and a clear message. Never approximate.
- **Output is part of the interface.** Messages, listings and debugger
  output are held to recorded transcripts; change them deliberately.
- **Comments explain why**, not what.
- **No em-dashes or double hyphens in prose**: comments, docs, commit
  messages. Use commas, colons or parentheses.
- **Keep it simple.** No dependencies without a strong reason. Add
  indirection only when a second user of it exists.

## Tests

When you change the ISA tables, the decoder, the machine's semantics,
or the assembler's layout or encoding, add a test rather than checking
by hand:

- **Behaviour a program can observe** (results, traps, paging, the
  console, how the assembler lays out or rejects a program) is a case in
  [`conformance/`](conformance/README.md). The format is checked
  strictly, so a misspelt expectation fails rather than passing.
- **The command's output** (listings, messages, run output, debugger
  sessions) is held to the transcripts in `rust/cli/tests/golden/`.
  After a deliberate change, run `DETRYTUS_UPDATE_GOLDEN=1 cargo test -p
  detrytus-cli --test golden` and review the diff before committing it.
- **Arithmetic, encoding and parsing details** get unit tests next to
  the code.

Then make sure the test can fail:

1. **Watch it fail.** Break the code your test guards and confirm the
   test goes red, then restore the code. A test never seen to fail is
   not a guard.
2. **Assert the precondition.** Check that the setup builds the
   situation the test exists for.
3. **Read registers and CSRs through their names** (`hart.reg("a1")`,
   `hart.csr("mtcause")`), not by indexing internal arrays.

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
