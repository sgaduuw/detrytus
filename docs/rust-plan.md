# Rust implementation plan

The Python emulator has done its job: it pinned the ISA down while a
real kernel (trylobite) was written against it. It is now the
bottleneck: even after the translation speed-up in sgaduuw/detrytus#8,
trylobite's demo (293,537 instructions, paging on) takes about 1.25 s
including start-up, and its 21 tests take 24 s (measured 2026-11-19,
down from about 60 s before #8). This plan
moves detrytus to Rust in steps that each leave a working tool, and is
shaped by one goal: **the end state is all Rust**. Python stays only
until each part has a Rust replacement that passes the same tests.

Decisions behind it are logged in `CONTEXT.md`.

## Principles

- **The core is plain Rust.** The machine (hart, MMU, bus, devices,
  image loading) is a library crate with no Python dependency. It is the
  part that survives unchanged.
- **Python bindings are scaffolding.** A separate thin crate exposes the
  core to the existing Python tools and tests while they still exist,
  and is deleted at the switchover.
- **Tests outlive their language.** Machine behaviour is tested by a
  conformance suite stored as data, assembly programs with expected
  results, which both implementations run. Python unit tests of Python
  tools retire with the tools; tests of machine behaviour migrate into
  the suite.
- **Two implementations are compared, not trusted.** As with the spec
  sync test, the Python and Rust cores run side by side and disagree
  loudly: the conformance suite on both, plus differential runs that
  compare full machine state.
- **The spec stays the source of truth.** Rust reads the same
  instruction table data where it can; where it cannot, a test parses
  `docs/isa.md` as `tests/test_spec_sync.py` already does.

## Layout

```
detrytus/                 the repository
  Cargo.toml              cargo workspace
  rust/core/              detrytus-core: the machine, no dependencies
  rust/py/                detrytus-py: PyO3 bindings, temporary
  rust/cli/               detrytus-cli: the Rust command line, grows
  conformance/            the conformance suite (data)
  detrytus/               the Python package, shrinking
  docs/, tests/, examples/
```

The bindings are a separate, optional package built with maturin
(`rust/py`), not part of the Python package: the owner chose that so
installing detrytus, and trylobite's CI with it, needs no Rust toolchain
until the switchover. `detrytus.machine.create(backend="rust")` selects
it.

## The conformance suite

Phase 1 built it: [`conformance/`](../conformance/README.md) holds the
cases and their format, and `detrytus/conformance.py` runs them on the
Python core. Each case is a program (inline, or files to assemble or
load) and its expected outcome and state, in TOML:

```toml
[[case]]
name = "a load needs no dirty trit"
program = '''
        .include "user.s"
        li    t0, 50 * PAGE
        lw    a2, 0(t0)
        sys
        .org  LEAF0 + 3 * 50
        .word LEAF + R + W + U + A + 60 * PAGE
'''
expect.exit_code = 4
```

The machine-behaviour tests that were Python (`test_cpu.py`,
`test_mmu.py` and the example runs) moved into it. The Rust core gets a
cargo test runner over the same files.

Until the Rust assembler existed, the Rust runner loaded `.dti` images
assembled by Python and committed under `conformance/build/`. Since
phase 5 it assembles the cases itself (`rust/conformance`), and the
images are gone. The suite now covers the assembler too: layout, local
labels, sections and every error message (`conformance/asm/`).

## Phases

Each phase is one or more pull requests, and each ends with everything
green.

1. **Conformance suite** (done). Create `conformance/` and its Python
   runner, and move the machine-behaviour tests into it as data. Python
   only; this pays off whatever happens next.
2. **Core crate: decode and integers** (done). Cargo workspace,
   `detrytus-core` with trit arithmetic, the instruction table, decoding,
   the integer and tritwise instructions, the bus and devices. Rust unit
   tests for the arithmetic; the conformance cases that need no traps
   pass. The Rust runner listed the cases it could defer to phase 3;
   anything unsupported outside that list, or wrong anywhere, failed.
3. **Core crate: privilege and paging** (done). Traps, CSRs, the timer,
   single step, the MMU and TLB with the same never-evicting behaviour.
   The whole conformance suite passes on Rust, with no deferral list
   left. Measured 2026-11-19 on trylobite's demo, three runs each,
   whole process: Python 1.14 to 1.20 s, Rust 0.066 to 0.082 s, with
   identical output and step counts. The interpreter is unoptimised
   (hash-mapped pages, a decode cache keyed by word), so that is a
   floor, not a ceiling.
4. **Bindings and differential testing** (done). `detrytus-py` exposes
   the core as the optional `detrytus-rs` package, so installing
   detrytus still needs no Rust toolchain; `detrytus.machine.create`
   (or `run --backend rust`) selects it. The conformance suite passes
   through the bindings, and so do trylobite's 21 tests (in 1.4 s,
   against 16.7 s on Python). A differential runner steps the
   conformance programs and 300 random programs on both cores and
   compares their whole state after every step; it runs in CI.
5. **Rust command line** (done). `detrytus-cli run` for `.dti` images
   (`rust/cli`, binary `detrytus`), then the assembler ported to Rust
   with the assembler tests moved into the conformance suite where they
   can be (`rust/asm`, and `detrytus asm`; `run` takes sources), then
   the debugger (`detrytus debug`, its stopping points driven by events
   the core records). Each tool switches over when it passes the same
   cases as its Python version: `tests/test_cli_parity.py` requires the
   same stdout, stderr, exit status and image bytes from both commands
   on every conformance program, the examples, a corpus of broken
   source and scripted debugger sessions.
6. **Switchover** (in progress). Rust becomes the default; the Python
   core, bindings and tools are removed once nothing depends on them.
   trylobite moves to the Rust command line. In three steps, as the
   owner agreed: (1) Rust is the default in the docs, and the debugger
   gains `xp` for reading physical memory, which trylobite's tests need
   once they can no longer reach into a Python machine; (2) trylobite's
   tests call the `detrytus` binary instead of the Python package, and
   its CI installs it with `cargo install` at a pinned commit; (3) the
   Python package and bindings are removed, the conformance suite
   becomes the main test of the Rust tools, and the parity and
   differential tests go with Python.

## Design notes for the core

- **Words are `i64`.** 3²⁷ ≈ 7.6 × 10¹² fits easily, and a full
  54-trit product fits in `i128`. Trit operations are integer
  arithmetic, as in Python.
- **The run loop lives in Rust.** No call into Python per instruction.
  Decoded instructions are cached, as in Python.
- **Debugging is native.** Breakpoints, watchpoints and trap catching
  are part of the core, which returns a stop reason. Calling back into
  Python on every access would cost the speed this is for.
- **Same observable behaviour, exactly.** Deterministic time, the TLB
  that never evicts, `tvec = 0` meaning unhandled, the device map: the
  conformance suite and the differential runner enforce it.

## Open questions

- Whether the instruction table should become a data file (TOML) read
  by both implementations, or stay in code with a sync test.
- Whether a JIT is worth it after the interpreter; decide by measuring
  trylobite on the Rust interpreter first.
