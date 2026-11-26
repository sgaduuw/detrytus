# CONTEXT.md

The decisions behind detrytus and why they were made. [`AGENTS.md`](AGENTS.md)
says what to do; this file says why things are the way they are.

Each entry gives the decision, the reasoning, and what was rejected.
Add an entry at the end when you make a decision that someone later
might reasonably want to undo: a spec change, a design trade-off, a
tooling choice that constrains future work. If a later entry reverses
an earlier one, say so in both.

Open questions about the ISA live in `docs/isa.md` section 13, not here.

---

## Project direction

### 2026-11-18 · A balanced-ternary machine, not a binary one

detrytus uses balanced ternary (trits of −1, 0, +1), following the Setun
and Setun-70. The appeal is the set of simplifications it allows: no
sign bit and no unsigned types, negation as tritwise inversion,
truncation that rounds to nearest, overflow-free comparison, and a
compare that has three outcomes. `docs/isa.md` section 2 lists them,
along with the costs (halving is not a shift; the outside world is
binary).

### 2026-11-18 · Target an operating system of its own, not Linux

Booting Linux was the first goal and was dropped. C itself assumes a
binary machine: C23 requires two's-complement signed integers, unsigned
integers must use a pure binary representation, and POSIX requires
8-bit bytes. The kernel leans on all of that (bitmaps, flag masks,
power-of-two pages). Running it would mean a binary-compatibility layer
over the ternary core plus a RISC-V-to-detrytus translator, which turns
detrytus into a binary machine with ternary storage.

Instead the ISA stays fully ternary and the goal is a microkernel OS
written for it, as a separate project. Consequence: there is no C
compiler, so the OS starts in assembly and probably wants a small
language of its own later.

### 2026-11-18 · Hardware virtualisation cannot help

macOS Hypervisor.framework and Linux KVM run guests of the host's own
instruction set only. No hardware runs ternary code, so detrytus is
always emulated. Speed comes from the emulator: a Rust port, a JIT, or
a QEMU TCG target, which would give a JIT, a gdbstub and device models
on both platforms.

## The ISA

### 2026-11-18 · 27-trit words and 9-trit trytes

The sizes are powers of three so every field boundary is clean. The only
powers of three near a word size are 9, 27 and 81. 81 trits is a
128-bit-class machine: an emulator would need big-number arithmetic
everywhere and page tables would need many more levels. 27 trits (about
42.8 bits) fits in a 64-bit integer, a full product fits in 128 bits,
and the address space of about 7.6 × 10¹² trytes is ample. A 36- or
41-trit word was rejected because it breaks the power-of-three sizes.

### 2026-11-18 · Centred memory layout, and memory as a ring

A word at address `a` covers `a-1`, `a`, `a+1`, and pages and tables
are centred the same way. Balanced fields are signed, so the low `k`
trits of an address are a signed offset from the nearest multiple of
3ᵏ: centring is what splitting an address by cutting trits gives,
without any adjustment arithmetic.

Address arithmetic wraps, so memory is a ring of 3²⁷ trytes. Because
3²⁷ is a multiple of every unit size, centred units tile the ring
exactly and none straddles the wrap point. A start-based word at the
highest aligned address would wrap round to the most negative one.

Centring is separate from byte order: the low trits sit at the lower
address, the ternary form of little-endian.

### 2026-11-18 · Signs and single trits instead of tables

Where a binary design needs a bit field or a lookup table, the ISA uses
a sign or a trit:

- the sign of a physical page number selects RAM (positive), the boot
  page (zero) or MMIO (negative);
- the sign of a trap cause separates interrupts from exceptions;
- privilege is one trit (−1 user, 0 kernel, +1 machine);
- a CSR number's top two trits encode its minimum ring and whether it
  is writable;
- a page-table entry's valid trit is `0` invalid, `-` pointer, `+` leaf;
- a branch hint, an `aq`/`rl` pair, a vectored trap mode are each a trit.

### 2026-11-18 · One profile, no optional extensions

Every instruction is mandatory. Optional extensions fragment software
(RISC-V's profiles exist to undo that), and a single-implementation
project gains nothing from them. The vector extension is the planned
exception.

### 2026-11-18 · One instruction per major opcode

Each instruction has its own 5-trit opcode (243 available), with no
secondary function field except `tlut`'s truth table. Decode is one
table lookup. Opcode 0 is permanently illegal so execution that runs
into zeroed memory traps.

### 2026-11-18 · `tlut` covers all tritwise logic

One instruction takes a 9-trit truth table and so expresses all 19,683
two-input trit functions, in the spirit of AVX-512's `vpternlog`. AND,
OR and the rest are assembler names for tables. `amotlut` does the same
atomically, replacing RISC-V's `amoand`/`amoor`/`amoxor`.

### 2026-11-18 · Integers and floats share one register file

There is no separate float register file, so a context switch saves 27
words and nothing more. That suits a microkernel, whose context switch
is on the hot path.

### 2026-11-18 · Float comparisons: Kleene predicates plus a total order

IEEE 754 has four relations (less, equal, greater, unordered) and a
trit holds three. `feq`, `flt` and `fle` return `+`, `-`, or `0` when a
NaN is involved, which is Kleene's unknown and maps C's negated
comparisons exactly. `fcmp` uses the IEEE total order, so it always
returns a trit and can feed `br3`. Version 0.2's `fcmp` returned 3 for
unordered; that was replaced because it was outside the trit range.

### 2026-11-18 · Compressed bundles address their second slot with the pc's low trit

A word with opcode +121 holds two 11-trit instructions. Full
instructions keep `pc` a multiple of 3, which leaves its low trit free;
`+` names slot B. Branch targets stay word aligned, and compressed
register fields reach all 27 registers. Variable-length instructions at
tryte granularity were rejected: they would cost a third of branch
reach and complicate fetch with centred words.

### 2026-11-18 · Memory model: RVWMO, simplified

One global order of memory operations, preserved-program-order rules
and three axioms, modelled on RISC-V's RVWMO. It is simpler in two
places: loads from the same address are always ordered, and only three
plain kinds of dependency carry order. Sequential consistency is a
legal outcome, so a one-instruction-at-a-time emulator conforms without
modelling buffers.

### 2026-11-18 · Three-level paging with a one-line root table

Pages are 3⁹ trytes; mid and leaf tables fill a page each; the root
table has 9 entries and fits a single cache line at the centre of the
page `ptbr` names. The first design used a 10-trit root index, which
made the root nine pages per address space, too heavy for a microkernel
with many processes. Accessed and dirty marks are set by the kernel on
a page fault, which keeps the walker simple.

### 2026-11-18 · Paging rules filled in from RISC-V (spec 0.4)

Implementing the walker showed that section 9 left the permission
rules, reserved encodings, superpage alignment, page-fault causes and
TLB caching unstated, and still called the walk "two-level". The gaps
were filled with RISC-V's answers, adapted to trits, because they are
well tested and a kernel written against them holds no surprises:

- loads need `R`, stores `W`, fetches `X`, and atomics both `R` and `W`,
  faulting as stores;
- ring 0 never executes user pages and reads them only with `SUA`;
- a superpage's physical page number must be aligned to its size;
- a pointer carries no permissions, a pointer at leaf level is invalid,
  and `-` in a flag trit is reserved, all of which fault, keeping those
  encodings free for later use as the instruction encoding does;
- page faults report the virtual address; walks read physical memory
  and may not read page tables from devices;
- a TLB may keep any successful translation until `fence.vm`, and a
  `fence.vm` that names an ASID leaves global entries alone;
- writing the reserved `ptbr` mode leaves `ptbr` unchanged.

These were proposed in the pull request that implemented paging
(sgaduuw/detrytus#3) and accepted by the owner as written.

### 2026-11-18 · The emulator's TLB never evicts

It keeps every successful translation until `fence.vm`. That is the
strictest legal behaviour, chosen so a kernel's missing `fence.vm`
shows up immediately rather than on a machine with a smaller TLB.

### 2026-11-18 · Trap banks per ring, named with `m` and `s` prefixes

Rings 0 and +1 each have their own trap CSRs, so the kernel handles
delegated traps without touching firmware state. The assembler names
them `mtvec`, `stvec` and so on, following RISC-V's convention.

### 2026-11-18 · Hypervisor support deferred

Ring +1 can host a hypervisor only by trap-and-emulate. Two-stage
translation is open question 3 in the spec, deferred because the OS
does not need it early.

## The emulator and assembler

### 2026-11-18 · Python first, Rust later

The ISA is still moving, and Python is quickest to change. The emulator
runs about 600,000 instructions per second (measured on a 600,000-step
loop: 0.95 s including startup, after the refactor). A Rust port is
planned once the design settles and the kernel needs the speed.

### 2026-11-18 · `isa.py` is the single source, checked against the spec

The assembler, disassembler and emulator all read one instruction table,
and a test parses the spec's tables to keep the two in step. Two
descriptions of one encoding drift unless something compares them.

### 2026-11-18 · Instruction semantics in an explicit table

`execute.py` maps each mnemonic to a handler, and the package refuses to
import if an instruction lacks a handler or vice versa. It replaced
looking up methods by constructed names (`"_op_" + name`), which failed
only at runtime and hid the calls from type checkers.

### 2026-11-18 · Code-quality gates

Strict mypy, ruff with a hand-picked rule set (not ruff's full set,
whose findings are mostly style opinions), complexity at most 10,
absolute imports only. Ruff's N818 (exception names ending in `Error`)
is switched off: `Trap`, `Halt` and `StepLimit` are machine events, not
errors.

### 2026-11-18 · Emulator behaviour outside the ISA

- `tvec = 0` means "no handler installed": the emulator stops with an
  unhandled-trap message. Address 0 is the reset vector, so no real
  handler lives there.
- `time` advances one tick per step, so runs are deterministic. `wfi`
  skips ahead to the earliest enabled timer, since a lone hart has
  nothing else to wait for.
- The decode cache is keyed by word value, so it can never be stale and
  `fence.i` is a no-op.
- Devices: the console at page −1, power-off at page −2
  (`docs/platform.md`).

### 2026-11-18 · CI action versions

CI pins `actions/checkout@v7` and `astral-sh/setup-uv@v10.3.3`, both on
Node.js 24. setup-uv stopped publishing floating major tags after v7,
so it has to be pinned to an exact release and bumped by hand.

### 2026-11-18 · The debugger lives in the emulator, not in GDB

GDB and LLDB know nothing of a ternary machine, and teaching them
(register layouts, disassembly, address arithmetic) is far more work
than a debugger of our own. `detrytus debug` has an engine with no user
interface, which tests drive directly, and a shell on the standard
library's `cmd` module. It reaches the hart through two hooks, one on
data accesses and one on trap entry, which cost one `None` check each
when no debugger is attached.

Inspection is kept free of side effects: memory reads go through the
page table without filling the TLB (`Hart.probe`) and never touch device
registers (`Bus.peek`). A debugger whose looking changes the program
would hide exactly the TLB and device bugs a kernel author needs it for.

The ISA's own debug triggers (spec 8.6) are separate: they are for the
OS's debugger and remain unimplemented.

### 2026-11-18 · Sections are placed explicitly; no linker yet

A section gets its address the first time it is used and never moves.
Letting one section follow the end of another would make addresses, and
so the sizes of instructions like `li`, depend on layout, which the
two-pass assembler is built to avoid. Separately built images share no
symbols; they agree on addresses by convention, typically constants in a
shared include file. A linker with relocations can come if that stops
scaling.

### 2026-11-18 · Images pack five trits to a byte

The `.dti` format stores trytes as packed trits, 5 to a byte (3⁵ = 243 ≤
256), which wastes about 5% instead of the 44% of a byte per tryte. It
carries labels and constants so the debugger can still name addresses.
When images are merged, overlapping trytes are an error, and a name
defined differently in two images is dropped rather than guessed.

### 2026-11-18 · Numeric local labels follow the GNU assembler

`1:` may be defined repeatedly; `1b` is the nearest at or before the
line (a label on the same line counts) and `1f` the nearest after. The
convention is familiar from other assemblers, and the assembler renames
each one to a unique `.L` name before layout, so nothing downstream had
to change.

### 2026-11-18 · Sections can run somewhere other than where they load

`.section NAME, LOAD, RUN` emits a section at its load address but lays
it out, labels included, at its run address, like a linker's separate
load and virtual addresses. trylobite's kernel needed it: loaded into
physical pages and run four pages lower, its labels were physical, so
code had to reach its own data pc-relatively and the debugger could not
name kernel addresses. The two addresses must differ by a multiple of 3
to keep words aligned in both. `.words N` arrived with it, for the
kernel's process table: unlike `.zero`, it starts at a word centre.

### 2026-11-19 · Translation caches what it has already checked

Paging made the emulator about six times slower: trylobite's demo ran in
3.1 s against about 0.5 s of work unpaged, because every access decoded
`ptbr` into a new object, re-extracted each flag trit of the leaf entry,
and built a `Trap` whether or not it faulted. Now `ptbr` is decoded when
written, a leaf's flags are parsed once, and the MMU keeps the result of
each permission check keyed by (ASID, page, access, ring, `SUA`). That
cache is derived from the TLB and cleared by every `fence.vm`, so it never
outlives a TLB entry and the never-evicting TLB's stale-until-fenced
behaviour is unchanged. Measured on trylobite's `demo.s`, three runs
each, same machine: 3.07 to 3.20 s before, 1.21 to 1.28 s after; an
unpaged loop is unchanged.

### 2026-11-19 · The implementation moves to Rust, built for an all-Rust end

Speed is now the limit on trylobite, and the design has held up under a
real kernel, so the emulator moves to Rust (`docs/rust-plan.md`). The
owner's aim is to end with Rust alone, so the plan is shaped to make
Python removable rather than permanent:

- **A pure Rust core crate**, with no Python dependency: the part that
  survives.
- **PyO3 bindings in a separate crate**, as scaffolding that lets the
  Python tools and tests drive the Rust core until each is ported, then
  deleted. A Rust core used only through Python would have left the
  tools and tests to rewrite at the end; a standalone Rust tool from day
  one would have meant porting the assembler and debugger before
  anything was usable.
- **A conformance suite stored as data** (assembly plus expected
  results), so tests outlive the language they were first written for.
- **Both cores compared**, by the suite and by differential runs of the
  full machine state, while both exist.

The work stays in this repository, and the Python core remains the
reference until the switchover.

### 2026-11-19 · Conformance cases are one TOML file per topic

The plan sketched a case as an assembly file plus a TOML file of
expectations. Most cases are a few lines, so a pair of files each would
have meant 176 files for what fits in eight; instead a TOML
file holds many cases, each with its program inline (or naming files to
assemble, as the example cases do), and shared assembly lives beside
them as includes. Choices that keep cases portable to the Rust runner:

- **The whole machine state is in the source.** The paging tests used to
  build page tables by poking memory from Python; the cases place them
  statically with `.org` and `.word` instead, through `paging/tables.s`.
- **Only observable state is checked**: outcome, console, ring,
  registers by name, CSRs (read with ring +1's privilege) and physical
  memory. Nothing about the TLB's or caches' insides; a stale
  translation is checked by what a load returns.
- **Strict format.** Unknown keys are errors and each case names exactly
  one outcome, so a misspelt expectation fails rather than passing
  unchecked.

Before deleting the Python tests, the suite was checked against 25
deliberate emulator bugs (permission checks, cache keys, fences, ASIDs,
trap entry, the timer, vectored mode). It caught all of them once four
cases were added; three of those bugs, vectored dispatch ignored, `PIE`
not saved, and an ASID-scoped fence flushing every ASID, had also passed
the old Python tests.

### 2026-11-19 · The Rust core's first shape (phase 2)

`rust/core` runs the unprivileged instructions: everything in sections
7.1 to 7.10 and the system instructions that need no CSRs. Choices made
on the way:

- **Devices are built into the bus**, not plugged in through a trait:
  the platform has two (console, power), and a trait object would cost
  an indirect call on every device access for flexibility nothing uses.
- **Memory is pages of `i16`, allocated on first write**, the Rust form
  of the Python core's sparse dict.
- **Without CSRs no trap handler can be installed**, so every trap is
  unhandled, exactly as in Python for a program that leaves `tvec` at 0.
  Instructions that need CSRs or rings (`csrr*`, `tret`, `wfi`,
  `fence.vm`) stop the machine as unsupported.
- **The Rust runner defers by list, not by luck.** Cases could come back
  unsupported only if they were named in `PHASE_3` (removed in phase 3);
  any other case that did failed, as did any wrong answer anywhere.
- **Images are generated by Python and committed**, with a Python test
  rather than a CI step checking they are current, so a stale image
  fails the same `uv run pytest` a contributor already runs.

The Rust side was checked as the Python side was, with deliberate bugs:
20 of 21 were caught after three new cases, `divr` with a negative
divisor, `tlut` with an asymmetric table (`tmin` is symmetric, so it
cannot tell its operands apart) and branches far enough to need every
offset trit. The one left, `instret` miscounted, cannot be seen without
CSRs and waits for phase 3.

### 2026-11-19 · The Rust core reaches the Python core (phase 3)

Traps, delegation, interrupts, the timer, single step, CSRs and paging
are in `rust/core` (`privileged.rs`, `mmu.rs`), and all 95 conformance
cases pass on it. Choices made on the way:

- **The entropy CSR follows the Python core's stream exactly.** Seeded,
  Python reads `random.Random(seed).getrandbits(48)`, a Mersenne
  Twister. The Rust core implements the same generator and seeding
  (`entropy.rs`, checked against values from CPython), so the two cores
  read the same "random" values and phase 4's step-by-step comparison
  needs no exception for them. The conformance suite still does not
  pin entropy values: they are a property of these emulators, not of
  the ISA.
- **The same TLB, cache and check order as Python**, including the
  checked-translation cache keyed by (ASID, page, access, ring, SUA),
  so stale-translation behaviour is identical.

The deliberate-bug check was rerun on the Rust side with the 25 Python
mutations plus 8 more (delegation, `tret`, single step, `ip`, CSR
privilege, `wfi`). Two got through, and new cases now catch both, in
Python as well: that only the software trit of `ip` is writable, and
that writing a read-only CSR is illegal. Only the control (entropy
fixed at 0) is uncaught, as intended.

trylobite's demo runs about 16 times faster on Rust: 0.066 to 0.082 s
against 1.14 to 1.20 s, three runs each, whole process, same output and
293,537 steps on both.

### 2026-11-19 · Bindings as an optional package, and the cores compared step by step (phase 4)

The plan had the Python package built with maturin, so that installing
it compiled the Rust core. That would have made a Rust toolchain a
requirement for anyone installing detrytus, trylobite's CI included.
The owner chose a separate, optional package instead: `detrytus-rs`
(`rust/py`, PyO3), installed with `uv pip install ./rust/py`.
`detrytus/rust.py` wraps it in the Python Machine's interface (load,
run, console, registers and CSRs by name, physical memory), and
`detrytus.machine.create` picks a core by argument or
`DETRYTUS_BACKEND`. The debugger and `--trace` need the Python core's
hooks and stay on it; the plan moves debugging into the Rust core later.

`tests/test_differential.py` steps both cores through the same program
and compares their whole architectural state after every step: all the
conformance programs and 300 random programs built from legal
encodings, with handlers that skip whatever traps. Run against
deliberate bugs that the conformance suite missed (in `cmpi`, `roti`,
`amomin`, `csrrtmin`, `lzt`, `instret` counting in the user ring, and
entropy), it caught all seven, and nine new cases now let the suite
catch them on its own. The one bug neither caught, capping a binary
right shift at 62 bits, cannot be observed: words never exceed 2⁴².

trylobite's 21 tests pass on the Rust core through the bindings, in
1.4 s against 16.7 s, run from a copy whose test helper calls
`create()`; trylobite itself still calls `Machine` directly.

`tests/helpers.py` is gone: its users were the CPU and MMU tests that
moved into the conformance suite.

### 2026-11-19 · The Rust command starts with `run`, held to the Python one's output

`rust/cli` builds a `detrytus` binary whose `run` takes `.dti` images
(sources need the assembler, still Python, and get a message saying to
make an image with `detrytus asm -o`). It copies the Python command's
behaviour exactly: console output streamed as the program runs, the
same `--ram-pages`, `--max-steps` and `--stats`, the same messages for
traps, step limits and overlapping images, and the same exit statuses
(the program's code, or 65, 66, 69, 70, 124, 2). Rather than a separate
set of Rust tests, `tests/test_cli_parity.py` runs both commands on
every conformance image and compares stdout, stderr and status, so the
two cannot drift apart unnoticed; it caught a one-character change to a
message when tried. The one allowed difference is the wording of an
operating-system error, such as a missing file, which each language
phrases its own way.

It runs trylobite's demo in about 0.06 s, against about 1.15 s for the
Python command.

### 2026-11-19 · The Rust assembler copies the Python one exactly

`rust/asm` ports the assembler, and the Rust `detrytus` gains `asm`;
`run` takes sources. The target was not "assembles the same programs"
but the same output for the same input: identical image bytes, listing
text, error messages and exit statuses, checked by
`tests/test_cli_parity.py` on every conformance program, the examples
and a corpus of 83 broken snippets that covers every error the
assembler raises. That meant reproducing some Python details on purpose:
`json.dumps` for image metadata, `repr()` quoting in messages,
`str.splitlines()` line endings, the order fields are checked in when
encoding, and `Path.resolve()` for includes.

Two things the comparison turned up. The Rust writer first left the
last packed byte unpadded, where Python pads with zero trits (each still
adds 1); the very last byte of every image differed. And `.align` with a
negative power silently made the Python assembler's address a fraction;
both assemblers now refuse powers outside 0 to 27 (3²⁷ already spans
every address).

With a Rust assembler the Rust conformance runner assembles its own
cases (`rust/conformance`), so the Python-built images in
`conformance/build/` and their regeneration step are gone. The
assembler's own tests moved into the suite where a program can show
them: `conformance/asm/` holds its error messages (a new `asm_error`
outcome, with `asm_error_line`) and its layout rules, run and observed.
What only a tool shows, the listing text and include files on disk,
stays in Python tests and the parity test. Deliberate bugs in the Rust
assembler were all caught by the suite or the parity test once three
cases and a snippet were added, except one that cannot be observed.

### 2026-11-19 · The Rust debugger: events from the core, output copied from Python

`detrytus debug` in Rust is a port of the Python engine and shell. The
plan put debugging in the core; what went in is small: when
`Hart::events` is on, each step records the instructions it executes,
the data accesses it completes and the traps it enters, and the
debugger reads that list to stop at watchpoints and caught traps and to
trace. The core also gained `probe` and `Bus::peek`, which read memory
without filling the TLB or touching devices. Breakpoints need nothing
from the core: the debugger checks the pc between steps.

As with the assembler, the target was identical output, so the shell
copies Python's `cmd.Cmd` line parsing and its `help` text (cleaned
docstrings and the column layout). `tests/test_cli_parity.py` runs ten
scripted sessions on five programs (one with two labels at an address,
one that crashes, one loaded from an image) and three interactive ones
over stdin, and compares every line. Ten deliberate bugs in the Rust
debugger were all caught once three of those were added.

Porting found two Python bugs, both fixed: `$r-3` was read as `$r`,
because the register pattern tried names before numbered registers, so
the documented form never worked; and `csr` with an unknown number let
the CPU's internal trap escape and crash the debugger, where it now says
`no CSR N`.

### 2026-11-20 · The switchover: how trylobite and installs move off Python

With every Python tool matched by a Rust one, phase 6 removes Python.
Two choices, the owner's, made on my recommendation:

- **trylobite's tests stay in Python but call the `detrytus` binary**
  as a subprocess (`asm`, `run`, and `debug` scripts for memory), rather
  than being rewritten in Rust. trylobite keeps pytest and needs no
  Rust code of its own.
- **detrytus installs with `cargo install`** (from a checkout or from
  git), with trylobite's CI pinning a commit. Prebuilt release binaries
  would be friendlier but need a release workflow; they can come later.

Done in three steps: Rust the default in the docs, trylobite moved,
then Python removed. The first step adds `xp`, examine-physical, to
both debuggers: one trylobite test reads a page table in physical
memory after the run, which it did through the Python machine object,
and `examine` reads through the stopped program's page tables instead.

### 2026-11-20 · Python removed; what replaced its tests

The last step of the switchover deletes the Python package, its tests,
the `detrytus-rs` bindings and the step-by-step comparison of the two
cores. Nothing they checked was dropped:

- **The command's output** is held to 171 golden transcripts
  (`rust/cli/tests/golden/`): listings, a corpus of broken source, run
  output and traces, debugger sessions, includes, merged files, a
  damaged image and illegal encodings. Each was recorded from the Rust
  command after checking the Python one printed exactly the same, so the
  parity the comparison tests enforced is now a fixed record.
- **Machine and assembler behaviour** stays in the conformance suite,
  whose format the Rust runner now checks as strictly as the Python
  runner did.
- **The spec** is checked by the Rust spec test, which gained the CSR
  numbers and all seven truth tables that only the Python test covered.
- **`run --trace`**, the one Python feature the Rust command lacked,
  was added first, and its output matched Python's on every conformance
  program.

The comparison of the two cores step by step has no successor: with one
implementation there is nothing to compare. The entropy source still
reproduces CPython's stream for a given seed, which no longer matters
but costs nothing.

Development now needs only a Rust toolchain; CI is one job of `cargo
fmt`, `clippy` and `test`.


### 2026-11-20 · Multiple harts: the proposal

ISA 0.5 (draft) describes machines with more than one hart; the
platform's hart controller is written down but not built. The choices,
for review:

- **One clock.** `time` is shared by every hart; `timecmp` stays per
  hart, so each has its own timer.
- **No private memory.** All memory is shared and coherent under the
  existing memory model. Per-hart data lives in RAM, found through
  `hartid`, `scratch`, `tp` or the start argument. A region every hart
  sees "at once" adds nothing that coherent RAM does not already give.
- **Mailboxes for fast messages.** One store queues a word for another
  hart and rings its software interrupt; nine deep, and a send that
  cannot be delivered is reported, never dropped silently. Big payloads
  go by pointer into shared RAM.
- **Threes.** Hart numbers are balanced and tree-shaped: the low trit is
  the place in a triad, the next the triad's place in a nine. Hart 0 is
  the centre and boots; other harts start stopped, so firmware does not
  park them. Group sends address a subtree. The reference platform
  allows 729 harts, one page of slots, with pages reserved beyond.
- **Hot-plug** is cooperative: the platform announces an added hart or
  asks for one to go; only a hart can stop itself, and only a stopped
  hart is removed.
- **Flushes stay local.** `fence.vm` and `fence.i` act on one hart;
  shootdown is done with messages, as on RISC-V.

Open: a wait-on-word instruction like Zawrs, and whether hart 0 may
ever be removed (isa.md 13).

### 2026-11-20 · Multiple harts: how the emulator runs them

The hart controller of platform.md is built. Choices the proposal left
to the emulator:

- **Rounds.** Each step, the clock advances once and every running hart
  takes one instruction, in hart-number order. Steps count rounds. This
  keeps runs deterministic and the clock shared, and with one hart it is
  exactly the old machine: every existing transcript and case passes
  unchanged.
- **`wfi` really waits.** With one hart, skipping time to the next timer
  was enough. With several, a hart that ran on after `wfi` would rarely
  line up with the others waiting, so time would never skip. A waiting
  hart now sits rounds out until an interrupt is pending and enabled in
  its `ie`; when all wait, time jumps to the next timer or hot-plug
  change, and if nothing could wake them, they return at once.
- **Interrupt routing.** The doorbell and hot-plug lines show in ring
  +1's `ip`, or ring 0's when `ideleg` says so, as a trap routed by
  `ideleg` would be.
- **Reservations live on the bus.** Any store by another hart to a
  reserved word breaks the reservation, so `lr`/`sc` loops are correct
  across harts. A hart's own stores leave it alone.
- **Stopped harts keep their last state** in the emulator, for tools to
  read, though the ISA says they hold none: a start replaces it.
- **Debugger.** It still shows and steps hart 0 only; selecting harts
  and hot-plug commands are the next step.

### 2026-11-20 · Debugging several harts

The debugger keeps the machine's rounds: a step runs every running hart
one instruction, so a session reproduces a run exactly. One hart is in
focus for inspection, and any hart can stop the run: a breakpoint,
watchpoint or caught trap on another hart moves the focus there, as an
all-stop debugger does. Stepping a single hart while the others wait
would be legal under the memory model, but would make a debugging
session diverge from `run`, which is worth more.

### 2026-11-21 · Waiting on a word: the proposal

ISA 0.6 (draft) proposes `wrs`, opcode −28 held, written beside the
atomics (7.9) but kept out of the opcode table until accepted, since a
table row and its implementation go in together. Multi-hart trylobite
showed the need: a hart waiting for the kernel lock spins on `amoswap`,
which on real hardware burns power and bus bandwidth, and in the
emulator burns steps.

- **Reservation, not value.** `wrs` waits on the word the last `lr`
  reserved, as RISC-V's Zawrs does. The alternative, "sleep while the
  word at rs1 equals rs2" (closer to a futex or x86's `umwait`), is one
  instruction instead of two, but needs a watched-address mechanism of
  its own. The reservation already is one: `sc` needs it tracked and
  broken by other harts' stores, and the emulator keeps exactly that on
  the bus. It also pairs naturally with `sc` for taking the lock once it
  comes free.
- **No lost wake-ups.** Check the value after `lr`; a store between the
  `lr` and the `wrs` breaks the reservation, so `wrs` returns at once.
- **Ends like `wfi`, plus a deadline.** An enabled pending interrupt
  ends the wait (so a user program cannot keep a hart asleep past the
  kernel's timer), as does a lost reservation, or `time` reaching `rs1`
  when `rs1` is not `r0`. One instruction with an optional deadline
  rather than Zawrs' pair (no timeout, short timeout); the register
  convention, `r0` for none, is the one `fence.vm` already uses.
- **The deadline stays**, at the owner's suggestion: a hart waiting on a
  busy holder can give up after a while and try other work, such as the
  next run queue's lock once trylobite has one per triad, or work to
  take from a neighbour. The timer could end the wait instead, but the
  kernel uses it for scheduling slices, and borrowing it for every
  bounded wait would mean saving and restoring `stimecmp` each time.
- **May be a no-op**, like `wfi`: correct code loops anyway, so a simple
  core need not implement waiting at all.

If accepted: the emulator treats a hart in `wrs` as waiting until its
bus reservation is gone, an interrupt is pending, or the deadline, which
also joins the moments the clock may skip to when every hart waits;
conformance cases cover each way out; trylobite's lock waits with `wrs`
instead of spinning.

### 2026-11-21 · `wrs` accepted and built

The owner accepted the proposal above; `wrs` is now in the opcode table
(−28) and the core. ISA 0.7. In the emulator a hart in `wrs` waits like
one in `wfi`, and also runs again once the bus no longer holds its
reservation or its deadline has come; deadlines join the moments time
skips to when every hart waits. With no reservation, `wrs` does not wait
at all. The `TW`-like trap bit stays open, now covering `wfi` too.

### 2026-11-21 · `csrw` from zero is refused (proposal)

A CSR instruction whose `rs1` is `r0` reads and does not write (isa.md
7.12), so the `csrw csr, rs` pseudo, which expands to `csrrw zero, csr,
rs`, turned `csrw csr, zero` into a read that changed nothing. It
assembled silently and cost trylobite two bugs: trap entry never
cleared `sscratch`, and a lock routine never cleared `sie`. The
assembler now refuses `csrw` whose source is `r0` under any name. The
raw `csrrw rd, csr, zero` stays legal, because it is the documented read
form (`csrr` expands to it) and its spelling shows `zero` in the source
slot. Only the pseudo is refused, because its name promises a write.
The other CSR instructions with `r0` as the source are left alone: no
pseudo hides them. This is a change to assembler syntax, so it is the
owner's call.

### 2026-11-21 · Console input from a file, fed up front

The core always had a console input queue, but only the conformance
runner filled it, so `run` and `debug` gave programs nothing to read.
trylobite's next milestone is a shell, which needs input it can be
tested with. `--input FILE` (on both, `-` for stdin with `run`) queues
the whole text before the first step, and the debugger's `input TEXT`
adds more by hand.

- **Up front, not live.** Reading a terminal key by key would need raw
  mode and would make a run depend on how fast someone types, which
  breaks the deterministic runs every transcript and test relies on.
  A program that finds no input sees −1, exactly as before.
- **`restart` queues the `--input` text again**, as it replays hot-plug
  changes, so a restarted session is the same run. Text from `input` is
  not kept: it was typed at a moment of the old run, and replaying it
  at the start would be a different run.
- **`debug --input -` is refused**: the debugger reads its own commands
  from stdin.
- **No escape for a trailing space in `input`.** The debugger trims its
  arguments, as Python's `cmd` did; `\s` writes a space instead.
- **Test the sign.** A code point is never negative, so `examples/echo.s`
  ends on any negative read with one `br3` (negative: done, zero: NUL,
  skipped, positive: a character) instead of comparing with −1. On a
  balanced machine −1 is an ordinary value, and a convention keyed to
  the sign leaves room for more than one "no character" later.

### 2026-11-21 · Device interrupts and a block device (proposal)

isa.md 12.3 listed an interrupt controller and a block device as still
to come. trylobite now has a shell whose `getc` can only report "none",
because nothing tells the kernel that input arrived, and it has no
storage for a file system. `platform.md` proposes both devices, marked
proposed until the owner accepts them. No ISA change: devices use the
external interrupt, cause −2, that the ISA already reserves for them.

- **A pared-down PLIC.** Claim and complete, one target hart per
  source, no priorities. Priorities and several targets matter when
  many harts compete for many devices; trylobite runs its kernel under
  one lock, so they would be unused weight. Claim and complete are kept
  because they let a handler serve every source in one entry and make
  level-triggered sources safe.
- **Level-triggered.** A source is pending for as long as its device's
  condition holds, so nothing is lost between claim and complete and
  there is no edge to miss when a source is masked.
- **Hot-plug stays where it is**, on the same cause through EVENT, so
  trylobite's hot-plug code is untouched. Folding it in as a source
  would give one dispatch path and can come later.
- **Console states named, compared by name.** A kernel that sleeps until
  a key arrives must also know when none ever will, or a shell at the
  end of its input sleeps forever. Console in reading −1 for both could
  not tell them apart. The proposal keeps −1 as `CONSOLE_EMPTY` and adds
  −2 as `CONSOLE_CLOSED`, using the room the sign convention left:
  programs test the sign and are unaffected, and only the kernel tells
  the two apart, by name. The owner asked that neither number be
  compared with directly. Rejected: a separate status register, which
  costs every read a second access.
- **A block is one page.** The kernel's allocator already hands out
  pages, and a transfer never needs an offset within a page. Smaller
  blocks (729 or 2,187 trytes) would waste less on small files.
- **Large transfers coalesce.** One command moves COUNT consecutive
  blocks through a scatter list of pages, and costs a fixed setup plus
  a little per block, so gathering a large write into one command pays,
  as on a real disk. The owner asked for this over one block per
  command. The list means the kernel never has to find contiguous
  physical pages.
- **Two bytes per tryte in the backing file.** Easy to seek and patch
  with ordinary tools. Because the device writes whole commands, a
  packed form (five trits per byte, about 10% smaller, like `.dti`)
  would gain little and lose that.
- **A fixed transfer time.** Deterministic runs, and a real wait for a
  kernel to fill.
- **The mailbox's "0 if empty"** is documented rather than changed:
  COUNT already tells an empty mailbox from a message of 0.

If accepted, in order: the controller and console states in the core,
with conformance cases for claim, complete, routing and masking; the
block device and `--disk`; then trylobite sleeps in `getc` and gains a
disk driver.

### 2026-11-21 · Device interrupts and console states built

The owner accepted the proposal above. The interrupt controller and the
console's two states are in the core; the block device follows.

- **The controller keeps no pending bits.** It holds ENABLE, ROUTE and
  which sources are in service; the bus asks the devices for their
  lines whenever a hart's external interrupt or a controller register
  is read. That is what level-triggered means, and it leaves nothing
  to fall out of step with a device.
- **`run` closes the console from the start**, behind whatever `--input`
  queued, since reads take the queue first. The conformance runner does
  the same unless a case sets `console_open`, which gives the
  debugger's behaviour; the debugger closes it on `eof`.
- **`eof`, not `close`**, as the spec named it. `EOF` (end of the command
  file, or Ctrl-D) still quits; commands are case-sensitive, as before.

### 2026-11-21 · The block device built

The last device of the accepted proposal.

- **The core touches no files.** It holds a `Disk` (blocks in, blocks
  out); the command line gives it the file and the conformance suite a
  disk in memory (`disk_blocks`), so the core keeps its no-dependency,
  no-I/O shape and every case is self-contained.
- **A command finishes at the start of a round**, before any hart
  steps, and the clock skips to that round when every hart waits, as it
  does for a timer. So a kernel that sleeps in `wfi` waiting for the
  disk costs nothing to emulate.
- **The list is read when the command starts and checked whole.** A
  word that names no RAM page, a list that is not word-aligned, or a run
  off the end of the disk fails the command at once and moves nothing.
  Memory with nothing behind it reads 0, which names no page, so a list
  pointed at a device fails without a separate check.
- **A read breaks reservations on its pages**, as a store would, so an
  `lr`/`sc` pair across a DMA read fails rather than succeeding on
  stale data.
- **The file is checked when it opens**, for an odd length and for
  values outside a tryte, so a bad image is a startup error rather than
  garbage in RAM later. A write error later is reported once on stderr
  and the run goes on, as a real disk's error would not stop the CPU.

### 2026-11-21 · The compressed table, set against measured code

`--mix` (`asm` for what is written, `run` for what executes) measured
trylobite's kernel and programs and its test suite against the first
table. ISA 0.8 adds four kinds of form, chosen by Eelco from the
instructions most often left without one:

- **`c.jal`** (op 13, CJ): calls were 7% of the static code, none
  compressible, and every call in a kernel fits ±3,280 words.
- **`c.beq`/`c.bne`/`c.blt`/`c.ble rd, rs, f`** (ops −1 to −4, CR):
  loop-closing branches compare two registers and jump back a few
  words. `f` is the offset, ±4 words. `bgt` and `bge` swap registers
  instead of taking opcodes. In the test suite these branches were a
  quarter of everything executed (page clearing and copying).
- **`c.sys` and `c.tret`**: new `f` values of op 3, no new opcode.
- **`c.lip rd, imm`** (op −5, CI): `li` of a page multiple, which is
  how device addresses and `PAGE` are loaded.

Together: 51% to 64% of trylobite's instructions compressible, and an
estimated 13% to 21% of its words saved by bundles. 8 of the 27 slot
opcodes stay free. Forms trylobite never uses were kept, since one
hand-written kernel is a narrow measure; a compiler's output may argue
differently. Left out: `lw`/`sw` with small offsets (about 1%, since
structure offsets are larger than 2 trits reach) and `la` (an `auipc`
and `addi` pair, 12% of the static code), which wants gp-relative
addressing rather than a slot.

### 2026-11-21 · `.assert` rather than `.error`

trylobite wanted a way for `include/kernel.s` to say "build shell.s or
demo.s" when assembled alone. The assembler has no conditionals, so a
bare `.error` would fire every time. `.assert E, "MESSAGE"` is one
directive instead of a conditional-assembly system: E is checked in
pass two, when every symbol is known, so `defined(NAME)` answers the
same wherever it is asked (and is refused outside `.assert`, where pass
one would see only the symbols so far). Comparisons joined the
expressions so a layout constant can be checked, as in
`.assert PCB_SIZE == 117, "..."`. Conditionals can still come if a
build ever needs to vary its code.

Unknown escapes (`\b`) became errors at the same time: they used to
stand for the character itself.

### 2026-11-21 · The mailbox's empty value (proposal)

RECEIVE read 0 from an empty mailbox, and 0 is also a message, so a
receiver had to read COUNT first whenever 0 could be sent. The
device-interrupts proposal documented that rather than changing it; it
was the one place left that broke the rule the console now follows
(no value is negative, named, and tested by sign).

A sentinel alone cannot fix it, since a message was any word. The
proposal gives up negative messages instead: SEND and GSEND refuse a
negative word (LAST says so), and an empty RECEIVE reads
`MAILBOX_EMPTY`, −1. Messages keep 0 to 3,812,798,742,493, far more
than a hart number, a page or a pointer into a kernel needs, and a
receiver takes a message with one load and a sign test. Keeping full
words with a second register that says whether the last RECEIVE took
one was the alternative; it costs a load on every receive. ISA 0.9.

### 2026-11-21 · What a slot links and where `c.sys` resumes (proposal)

Building the emulator's side of bundles showed three rules the 0.8
table left open, all from the same arithmetic: `pc + 3` is the next
instruction only for a full instruction. From slot B (`a + 1`) it is
`a + 4`, slot B of the following word.

- `jal` and `jalr` link "the next instruction": `a + 1` after slot A,
  `a + 3` after slot B, `pc + 3` as before for a full instruction.
- `sys` sets `tval` to the next instruction, and handlers resume there.
  Adding 3 to `tpc` is wrong for `c.sys` in either slot, and working
  it out from `tpc` would need the kernel to read the trapping word to
  tell slot A from a full `sys`. `tval` was 0 for `sys`, so kernels
  that add 3 keep working for full `sys`. Dropping `c.sys` was the
  other way out; it was added in 0.8 because system calls were among
  the most frequent uncompressible instructions.
- A `pc` ending in `-` traps as misaligned; one ending in `+` whose word
  is not a bundle traps as illegal with that word in `tval`; an illegal
  slot reports the whole bundle in `tval`, and `tpc`'s low trit says
  which slot. ISA 0.10.

### 2026-11-21 · Bundles run (ISA 0.10 built)

The hart runs a slot by expanding it to its full instruction and
decoding that, so a slot has no behaviour of its own to get wrong: the
one table in `compressed.rs` both expands slots and says which slot a
full instruction fits, and a unit test walks all 177,147 slot values to
check the two agree. What a slot changes is addressing alone: the next
instruction (`a + 1` or `a + 3`), the base for branch offsets (the
bundle), and `tval` for `sys`. A debugger's trace shows a slot as the
full instruction it ran. The assembler does not write bundles yet, so
the conformance cases build them with `.word`.

### 2026-11-21 · `c.` mnemonics: bundles written by hand

The assembler takes the `c.` mnemonics as a first step towards writing
bundles, before any automatic pairing. Each takes the operands of its
full instruction less the ones its form fixes, so `c.lwsp a0, 6` means
`lw a0, 6(sp)` (the field holds 2) and `c.lip a0, 2 * PAGE` the value
loaded, not the field: offsets and values read the same as in the full
instruction, and a branch names its label. The assembler encodes the
full instruction and asks the compressed table which slot it is, so
the form's limits live in one place, and a unit test reassembles the
disassembly of every slot value. Two in a row share a word; a label or
any other statement between them starts a new one, a rule simple
enough to see in the source.

### 2026-11-21 · Automatic bundling with `.compress`

Eelco chose directives over a command-line flag: the places that must
stay full width (vectored trap tables, size-checked blocks) are known
in the source, and the build command stays the same. Under `.compress`
the assembler pairs neighbours by the rules `c.` instructions already
follow, and leaves a lone instruction full, since a `c.nop` filler
saves nothing. Short branches make it a fixed point: layout runs with
every candidate allowed into a slot, drops the paired ones whose full
instruction does not fit where they landed, and runs again. The set
only shrinks, so it ends, usually in two or three rounds; an
instruction once dropped is not retried, which can only cost a few
words. A trial build of trylobite with `.compress` around its kernel
and programs (and its syscall return moved to `tval`) went from 1,598
to 1,314 words, 18% smaller, data included, and passed its tests but
one that searched the debugger's output for the letter c.

### 2026-11-21 · gp-relative addressing (proposal, ISA 0.11)

After bundling, `la` (an `auipc` and `addi` pair) was the largest share
of trylobite's code that no slot could take, about 12% of it. No new
full instruction is needed: `addi`, `lw` and `sw` already carry a
16-trit offset, ±21 million trytes, so with `gp` pointing into a
program's data, `la` is one `addi` from `gp` and a global is one load
away. The register was already named `gp` in 4.2; the spec now says
what it means: set once, pc-relatively, never changed, so no function
saves it, and a trap handler with a `gp` of its own swaps it the way it
swaps `sp`. Where it points is left to the program, as `sp` is.

The assembler learns where `gp` points from `.gp E`, which belongs to
a section rather than running on from where it is written. A trial
with a lexical directive broke at once: trylobite's tests append
programs after the kernel's include, and those inherited the kernel's
`gp`. `la gp, ...` stays pc-relative, since it is the instruction that
sets `gp`. The assembler cannot check that the register holds E, so it
takes it on trust, as it takes a section's run address.

Three slot forms came with it, each counting words because globals are
words: `c.lagp rd, imm` (op −6, `addi rd, gp, 3·imm`), `c.lwgp` (op −7,
`lw rd, 3·imm(gp)`) and `c.swgp` (op −8, `sw rd, 3·imm(gp)`). A local
prototype on trylobite (kernel globals reordered so the small ones sit
next to `gp`, each process started with `gp` at its base): 977 code
words before, 910 with `gp` and no new form, 886 with `c.lagp`, and 880
with all three and `la` + `lw` pairs folded into `lw rd, symbol`. The
load and store forms add only 6 words today, but folding without them
loses the `c.lw`/`c.sw` the pair had, so Eelco took all three: folded
code takes an instruction and a register fewer and stays compressible.
5 slot opcodes stay free.

### 2026-11-21 · gp-relative addressing built (ISA 0.11)

The assembler handles `.gp` in a pass of its own (`rust/asm/src/gp.rs`)
between parsing and layout: it follows sections, and rewrites `la` and
a load or store that names a symbol into the one machine instruction
each stands for, an offset from `gp` written as `(symbol) - (E)`.
Layout then sees an ordinary one-word `addi`, `lw` or `sw`, so automatic
bundling needs no change to put it in a `c.lagp`, `c.lwgp` or `c.swgp`
slot, and the listing still shows the line as written. The rewritten
statement remembers its symbol and E, so a distance beyond 16 trits is
reported in those terms rather than as an immediate that does not fit.
The three slot forms are rows of the compressed table like the others;
the emulator runs them through the same expansion.

### 2026-11-22 · A live console for `run`

- **So a kernel can be used, not only replayed.** trylobite's shell
  waits at its prompt for input, but `run` closed the console before the
  first step, so a session ended at the end of `--input`. At a terminal
  (stdin and stdout both one) without `--input`, or with `--live`, a
  thread now reads stdin and the bytes reach the console as they come;
  the end of the pipe, or Ctrl-D at a terminal, closes it. Both
  terminals, not stdin alone, so that a test harness that captures the
  output never gets a live console by accident and waits for a key.
- **The terminal passes every key** (`stty -icanon -echo -isig -iexten
  -ixon`, put back when the run ends): the guest echoes, edits and acts
  on Ctrl-C itself, as on a serial line. Output processing stays on, so
  a newline still returns the carriage. stty rather than a termios crate
  keeps the command free of dependencies; without stty the run goes on
  with the terminal as it is. Ctrl-] stops the machine, as telnet's
  escape, since Ctrl-C now belongs to the guest.
- **No step limit, and no spinning while it waits.** A live run has no
  default `--max-steps`. When every running hart waits with nothing in
  the machine that could wake one (no timer, no block command, no
  hot-plug change), the machine has stalled: `run_until_stalled` stops
  the chunk there, and the command sleeps up to 20 ms for a key before
  running on. Typing is answered within a chunk of 10,000 steps. A guest
  that keeps a timer running while idle never stalls and still costs
  CPU; trylobite turns its timer off while idle.

### 2026-11-23 · A live console paces idle time

- **So a guest that sleeps sleeps.** trylobite's `top` redraws every
  million ticks with a `sleep` that sets a timer. With nothing else to
  run, every hart waits for that timer and the machine skips the clock
  straight to it (Time, platform.md), so on a live console top redrew as
  fast as it could draw. A live run now limits each skip to 20,000 ticks
  (`Machine::set_pace`) and waits 20 ms, for a key, between them: about
  a million ticks a second of idle time. The machine stops a chunk at a
  paced skip as it does at a stall (`take_paced`).
- **Only idle time is paced.** Running, the clock still moves one tick
  per step, faster or slower than a million a second depending on the
  host, so times are only roughly seconds. Tying `time` to the wall
  clock altogether would make runs depend on the host even without a
  live console; a run with `--input` still skips at once and stays
  deterministic. A skip shorter than 20,000 ticks is not slowed at all,
  which covers the disk: a transfer takes 1,000 ticks plus 100 a block.

### 2026-11-23 · Trice, a compiled language, and an ABI

The owner asked for a design for the first compiler. Standard C and
Rust assume binary integers (the first entry above), so the first
language is a C-shaped one built on the machine's own types:
`docs/trice.md`, with the calling convention and data layout it
compiles to in `docs/abi.md`. Both are specs only; the compiler comes
next. The owner settled these choices:

- **The name is Trice** (sources `.tc`, `detrytus tc`). "C3", the
  owner's first idea, belongs to c3-lang, and "Tern" to tern-lang.
- **A boolean has three values**: true (+), false (−) and unknown (0),
  for "not decided" or "not done yet", as isa.md 4.2's boolean trit
  already allows. `&&` and `||` are Kleene AND and OR (`tmin`, `tmax`).
  Conditions must be booleans, with no "nonzero is true": under C's
  rule a stored false (−1) would read as true. `if` and `while` branch
  only on true, as SQL's `WHERE` treats `NULL`, and a `which` statement
  (one `br3`) handles all three cases.
- **Trit-wise operators replace bitwise ones**: `&` is `tmin`, `|` is
  `tmax`, `-` is the only negation (`not` and `neg` are one
  instruction), `<<` and `>>` shift by trits, and `<=>` gives a trit.
  The binary operations stay builtins, for binary data.
- **Overflow wraps**, as the ISA's `wrap`, rather than being undefined.
- **No libc.** The compiler gives only a `sys` builtin; start code and
  helpers are ordinary files a program imports (trylobite's `lib/`).
  What languages share is the operating system's interface, not a C
  library: a later Rust target starts `no_std` over a crate of system
  call wrappers and ports `std` onto the calls directly, so a libc
  would only pull the kernel's API toward C.
- **The compiler is Rust, in this workspace** (`rust/tc`), not a new
  repository: it reuses `rust/asm` to read `.equ` constants from
  imported assembly (so `include/layout.s` stays the one place for
  trylobite's numbers) and to assemble, and trylobite's
  `tools/install-detrytus` installs it with no new pin. It knows the
  ISA and the ABI but no system call.
- **One assembly file per program** until there is a linker. The ABI
  keeps Rust in view: `a0`/`a1` two-word results (slices), builtins
  named after instructions so a `core::arch` module can mirror them,
  and an object format and linker to come, which Rust needs too. Rust
  here will always be binary Rust (a byte per tryte, `i64` in two
  words), so Trice stays the native language.

The first milestone is trylobite's `bin/echo` rewritten in Trice,
passing its tests unchanged; then `bin/wc`.

### 2026-11-23 · The first Trice compiler walks the tree

- **Correct before small.** `rust/tc` checks types as it generates
  code, straight from the tree: values in `t0` to `t6` as a stack,
  locals in frame slots reached from `s0`, the temporaries a call must
  keep pushed around it. trylobite's `bin/echo` comes out at about 80
  words against 21 hand-written. The IR and register allocation of the
  design's milestone 5 are what shrink it; until then the output is
  meant to be obviously right, and easy to read beside its source
  (each statement carries a `# file:line` comment).
- **Comparisons as values need no branch.** `x > y` as a bool is
  `sign(2·cmp(x, y) − 1)` and `x >= y` is `sign(2·cmp(x, y) + 1)`:
  `cmp`, `add`, `cmpi`, with no label, so the assembler can still
  bundle around it. In conditions, comparisons branch directly.
- **Imported assembly is read by the assembler's own parser** for its
  `.equ` values, so a trylobite program writes `SYS_WRITE` and the
  output still says `li a0, SYS_WRITE`; the file is `.include`d where
  the import stands, by a path relative to where the output is
  assembled.

### 2026-11-23 · What porting trylobite's tools asked of Trice

- **The ports came first.** trylobite's wc, grep, head and upper were
  rewritten in Trice before the compiler grew, to see what real
  programs need. They needed none of the features the first compiler
  left out. What they did need was for unused helpers to cost nothing:
  every program imports trylobite's `lib/io.tc` whole, so `upper` carried
  all of it.
- **Unused functions are left out, after checking.** The compiler
  follows calls from `main`, and counts a function that an `asm` block
  or imported assembly names (an operand of an instruction, not a
  comment) as reached, since it cannot see what assembly calls. An
  unreached function is still compiled, so its errors are reported,
  and then dropped with its strings. With no `main` everything is kept.
- **Structs follow abi.md 1.2 exactly**, so a struct global with an
  initialiser is emitted as whole words: the trytes of a word-aligned
  struct are packed three to a word, low tryte first (isa.md 5.2),
  because a label names a word's centre and a leading tryte field sits
  below it. Struct types are kept by name, with layouts in a registry,
  so a struct can point to its own kind.
- **Struct assignment copies**, word by word (or tryte by tryte), and
  as a loop past nine words. Struct values in calls and results stay in
  section 11's list; a function takes or fills a struct through a
  pointer.
- **A call with more than six arguments stores each argument as it is
  evaluated**, the seventh onward where abi.md 2.2 puts them and the
  first six above, then loads `a0` to `a5`. Evaluation stays left to
  right and needs one temporary however many arguments there are.

### 2026-11-23 · Registers before an IR

- **Rules, not an optimiser.** Register allocation proper needs an
  IR. Most of what it would win in these programs comes from a few
  rules the tree walker can follow. Scalar locals go in `s1` to `s8`,
  and leaves keep their parameters in `a0` to `a5`. Variables and
  constants are used in place, with no copy to a temporary. A frame
  pointer is kept only when something is in memory, and with `.gp`
  globals are reached by name. trylobite's `wc` went from 1,618 trytes
  to 1,111 (659 hand-written), and `echo` from 247 to 220 (70).
- **Order stays left to right.** An argument that is a variable is put
  in its register after the arguments that need code, so it must not
  change in between. A register variable is moved early only when no
  other argument assigns to it, and a variable in memory only when no
  other argument calls or assigns. A comparison or sum copies a
  register variable first when its right side assigns to it.
- **A trit in memory is cut to a trit.** `st` keeps nine trits, so a
  `trit` stored from a wider value needs `tex` first. That was missing
  in the first compiler, and so was cutting arguments and results to a
  `tryte` or `trit` parameter or return type. abi.md 2.2 says the
  value travels as itself, so the caller cuts it.
- **Test programs run twice**, with and without a global pointer
  (`tests/bare_gp.s`), and must agree.

### 2026-11-24 · An IR for Trice: assembly over virtual registers

- **The IR is the machine's own instructions.** Code generation emits
  detrytus assembly over virtual registers (`%n`), and the back end
  (`rust/tc/src/alloc.rs`) works on that: liveness, dead code,
  reuse within a block, colouring with coalescing, spills, peepholes,
  frames. A separate IR with its own operations would have meant a
  second instruction selection for no gain on one target. A call says
  how many argument registers it reads (`call f, n`), which is all the
  allocator needs to know of the ABI.
- **Ready for another front end.** Nothing in the back end is Trice's:
  it takes lines of assembly and a few facts about the function (does
  it return a value, how many frame words its locals use). A Rust
  backend (rustc_codegen) would emit the same lines and reuse it.
- **Size, not speed, sets the rules.** Values are not kept over a call
  (a saved register costs two instructions); a function is copied in
  only when that cannot grow the program much. The compressed forms
  shaped the colouring: `rd = rd op rs` fits a slot, so a result
  prefers its operand's register, and a label splits bundles, so
  labels nothing jumps to are dropped. trylobite's `wc` went from 1,111
  trytes to 901 (659 hand-written).
- **A system call changes only `a0` to `a5`.** trylobite's trap entry
  saves and restores every register, as preemption requires, so values
  in temporaries live across `sys`. trice.md 8 states it as what the
  compiler assumes of any operating system.
- **Pointers may be devices.** Only loads of globals by name are
  reused; a load through a pointer is always made, so polling a
  device's register works.

### 2026-11-24 · Objects and a linker

Separate compilation arrived for what a Rust backend will need, and so
that Trice programs can stop compiling their whole library each time
(`docs/linker.md`). The owner approved the work; these choices were
proposed with it:

- **Objects are text.** One line per section, run of trytes,
  relocation and symbol, under a versioned header. A program is a few
  thousand trytes, so size does not matter yet, and text can be read,
  compared and recorded in tests without a tool. A packed form like
  `.dti`'s can follow if it ever does.
- **The assembler makes objects itself** (`asm -c`), rather than
  objects carrying source for the linker to assemble later. Values are
  evaluated as a constant plus whole multiples of bases (sections the
  linker places, external names), and anything with a base left over
  becomes a relocation. Without `-c` the bases never appear, so whole
  sources assemble exactly as before.
- **Five relocation kinds** (a word, an `imm` field, a branch or jump
  offset, and the `lui` and `auipc` pairs) cover every place the
  assembler puts a value, and a relocation adds one base and subtracts
  one, which covers pc-relative values and offsets from `gp` across
  objects. A pair is one relocation, not two: `li` and `la` always emit
  both words together, and the balanced split needs no carry.
- **Placement stays explicit.** The 2026-11-18 entry kept addresses out
  of layout; a linker can let sections follow one another, since sizes
  are settled by then, but an address is still given once, in a source:
  the object that gives a section an address places it, the others'
  parts follow, and an unplaced section follows the one before. No
  linker scripts and no addresses on the command line. trylobite's
  `lib/start.s` already places every program at `V_USER`.
- **No relaxation.** An instruction the linker finishes is never put
  in a compressed bundle, since its reach is unknown when the bundles
  are chosen. A linker that shrinks code after placing it could win
  those slots back; it is not worth its complexity before there are
  linked programs to measure.
- **Undefined names are external**, as in other assemblers, so a typo
  surfaces at link time, at the line that used it.
- **Trice compiles a file on its own** (`tc -c`, trice.md 13) by
  treating what it imports from other `.tc` files as declarations, and
  making everything it defines global. Nothing is dropped as unused,
  since another object may call it. Imported assembly is included only
  by the file that imports it, so start code lands in the program's
  object alone. trylobite's `echo`, linked from `bin/echo.tc` and
  `lib/io.tc`, runs unchanged; it is 339 trytes against 154 compiled
  whole, from the library functions it does not use and the
  instructions the linker finishes, which stay out of bundles.

### 2026-11-24 · Unused code left out at link time

A Rust backend will emit large libraries of which a program uses
little, so the linker now leaves out what nothing uses, as
`--gc-sections` does elsewhere (`docs/linker.md`). The owner chose it
as the next step after the linker; these choices were proposed with it:

- **A part, not a section per function.** `.part` starts an anonymous
  piece of the current section, written in the object as `text:2` and
  so on, rather than asking each function for a uniquely named section.
  Placement is unchanged: a part is one more contribution to its
  section, in source order.
- **What is before the first `.part` is a root**, along with every
  section given an address. Start code needs no marking and plain
  assembly objects behave as before; only code that opts in can be
  dropped. There is no entry-symbol option, matching the rule that
  layout lives in sources.
- **Reachability follows relocations**: a part's own base, or a global
  name to the part that defines it. A part reached only through a
  computed address it never names would be dropped, as with other
  linkers; nothing in detrytus or trylobite does that.
- **Trice parts every function and global** under `tc -c`. Calls
  within a file now cross parts, so they are relocated and stay out of
  bundles; relaxation would win them back. Linked `echo` drops from 339
  to 189 trytes, against 154 compiled whole.

### 2026-11-24 · Linked Trice programs close the gap

Linked trylobite programs were 6 to 23% larger than whole ones. The
owner asked for linker relaxation next; measuring first showed most of
the gap had cheaper causes, fixed here before deciding on relaxation:

- **Imported assembly is a part.** A library's assembly (trylobite's
  `lib/fs.s`) sat before its first `.part`, so every program kept it.
  Now `tc -c` starts a part before each `.include`; placed sections,
  such as a program's start code, are kept as before.
- **A program is compiled as a whole program**, apart from its imports.
  A file defining `main` is called into only by its start code, so it
  exports only `main` (start code may be its own object), drops what
  nothing calls and inlines single callers itself, and its calls are
  ordinary assembled branches that bundling can take.
- With trylobite giving `lib/io.tc` the `.gp` its programs have, linked
  echo is 153 trytes against 154 whole and upper 366 against 375; wc
  957 against 901, grep 1,713 against 1,663 and head 891 against 851.
  What is left is calls into the library, which the linker finishes and
  so are not bundled, and library functions not copied into callers.

### 2026-11-24 · Relaxing linked sources

The owner chose to build relaxation for what the entry above left:
calls into a library, which the linker finishes and so were never
bundled. Choices:

- **Relax by assembling again, not by moving code.** A linker that
  shrinks code itself must know every branch and every `end - start`
  inside a section, which objects settle as constants; recording them
  all would change the format throughout. Instead `link`, given
  sources, assembles them again with the addresses its first link
  found, so the assembler pairs what would fit, and everything within
  a section is laid out afresh by the assembler as always. Objects
  given ready-made are linked as they are.
- **The linker checks every slot.** Two relocation kinds, `slot.a` and
  `slot.b`, put a value into a slot's full instruction and encode it
  again. One that no longer fits where the program landed sends that
  instruction (by file and line) back out of slots, and the sources are
  assembled again; the set only grows, so this ends, as the
  assembler's own pairing loop does. A success with new addresses tries
  again while the program shrinks.
- Linked wc is 933 trytes against 901 whole (957 before), grep 1,689
  against 1,663, head 864 against 851; echo 147 and upper 360 are below
  their whole builds. What is left is library functions not copied
  into their callers.


### 2026-11-24 · Rust through LLVM IR

The owner wants Rust programs on detrytus. A first step runs a
`no_std` Rust program on the bare machine and on trylobite
(`docs/rust.md`). Choices:

- **Translate rustc's LLVM IR, not write a backend.** rustc compiles
  for a built-in 32-bit target and writes textual IR for the program
  and for core; `detrytus ll` reads it, keeps what `main` reaches and
  lowers it to the virtual-register IR Trice's back end already
  allocates. Rejected: a rustc codegen backend, which needs nightly
  compiler internals and breaks with each nightly; WebAssembly, which
  is stable but loses SSA form and needs a base added to every memory
  access; translating RISC-V machine code, which loses the types, so
  every operation would wrap; and a real LLVM backend for detrytus,
  far too large. The IR's text format can change with LLVM versions;
  the reader skips what it does not use, which keeps it tolerant.
- **Stable Rust.** `-Zbuild-std` (to get core's IR) and
  `-Cpanic=immediate-abort` (so that panics do not pull in
  formatting) are unstable, and `RUSTC_BOOTSTRAP=1` allows them on a
  stable toolchain. That keeps the build to `rustup target add` and
  `rust-src` rather than a pinned nightly.
- **Binary semantics, faithfully.** A byte is a tryte holding 0..255
  and an N-bit integer is held as its unsigned value, wrapped with
  `band` after anything that could leave the range; the binary
  operations of isa.md 7.6 do bitwise work. Keeping a `u32` in one
  word in memory instead of four trytes would be smaller and faster,
  but LLVM freely reads integers as bytes and bytes as integers, so
  that is left until it can be shown safe.
- **Whole program, as one assembly file.** A pointer in static data is
  four trytes computed from a label with `/` and `%`, which only the
  assembler can work out; the linker's relocations are linear. So the
  output is a whole program rather than an object.
- **Not yet:** integers over 32 bits (and so `core::fmt`), floats,
  atomics, more than six argument words. Each is refused by name.
  (64-bit integers and long calls came on 2026-11-25, below.)

### 2026-11-25 · 64-bit integers, and with them core::fmt

The owner approved 64-bit integers next, since `core::fmt` needs them
(`docs/rust.md`, "64-bit integers"). Choices:

- **Two words, low 32 bits first.** A word holds about 42 bits, not
  64. A `u64` becomes two scalars, so everything that only moves
  values (phis, loads, stores, calls, struct fields) already works,
  and only the operations need code of their own. Rejected: one word
  plus an overflow word of other widths, which would make every
  operation a special case.
- **Only what core::fmt needs, refused by name otherwise.** Division
  is by a constant below 2²⁵ (core divides by 10 000 and 100), a
  16-bit digit at a time; shifts are by a constant. A general 64-bit
  divide and variable shifts wait for a program that needs them.
- **Binary alignment where LLVM asks for it.** `fmt::Arguments` keeps
  a flag in a pointer's low bit, so alignment is not a nicety: static
  data gets the assembler's new `.balign N` and aligned allocas a
  base register rounded down at run time with `band`, since the frame
  pointer's value is not known to be aligned. Rejected: keeping every
  frame aligned to 24 trytes, which would touch Trice's frames too and
  waste stack in every function.
- **More than six argument words** go on the stack as abi.md 2.2 says
  (core calls `pad_integral` with eight), and an `alias` is another
  name for what it aliases.

### 2026-11-25 · Smaller Rust programs

The owner approved making Rust programs smaller next, suggesting a
`u32` kept in one word. Choices (`docs/rust.md`, Size):

- **Memory stays a byte a tryte.** LLVM reads integers as bytes and
  bytes as integers (struct copies, a `u64` read as two `u32`s, core's
  byte counting), so a `u32` cannot live in a word in memory without
  proving no such access reaches it, which the IR does not make easy.
  In registers it already is one word.
- **Helpers for 2- and 4-byte loads and stores** instead: a `jal t6`
  to code emitted once, with the address in `t5` and the value in
  `t4`, changing only `t2`..`t6`, which the allocator is told, so the
  rest of the registers survive. That gave most of the saving (rfact
  5 715 to about 4 100 trytes). Rejected: ordinary calls, which would
  make every load spill the caller's temporaries.
- **64-bit helper functions written in LLVM IR** (`lower/wide.ll`),
  lowered like the program and kept only when used. They also lift
  the limits of the first version: any divisor, signed division and
  shifts by a variable amount now work.
- **A bug found on the way:** a 32-bit `shl` by 10 or more could
  overflow the word before its result was masked, since `bsh` wraps
  in ternary. A shift now drops the bits that would leave the type
  first.

### 2026-11-25 · Atomics through one lock

The owner asked for the remaining Rust gaps: atomics, 128-bit integers
and floating point. Atomics first (`docs/rust.md`, Atomics):

- **One lock word per program, every atomic operation done plainly
  under it.** An atomic `u32` is four trytes in memory, a byte a
  tryte, while `amoswap`, `amoadd` and `cas` work on aligned words,
  so the native instructions cannot do a 32-bit atomic add. The lock
  makes atomics atomic against each other on any number of harts,
  which is all Rust promises. Rejected: keeping atomics in words
  (the same objection as for keeping a `u32` in a word: code reads
  their bytes, `AtomicU32::as_ptr` and `from_ptr` hand them out), and
  ignoring atomicity on the grounds that trylobite runs one hart
  (true today, but kernel code and future SMP would break quietly).
- **The expansion happens when the IR is read**, into loads, stores,
  `icmp`, `select` and `insertvalue` between two new operations, lock
  and unlock, so the lowering already handles everything else.
- **A `fence` is nothing**: a hart sees its own accesses in order,
  and the lock orders those of others.

### 2026-11-25 · 128-bit integers as limbs

Second of the remaining Rust gaps (`docs/rust.md`, Wide integers):

- **The two-word code became N-limb code** rather than a separate
  128-bit path: one limb per 32 bits, the same carry, borrow, column
  multiply, digit division and limb-weighted comparison for any width
  up to 256 bits. 64-bit code comes out the same size as before
  (rfact still 3 960 trytes).
- **256 bits, not 128**, because core formats a `u128` by a 256-bit
  multiply by a constant and a shift, its way of dividing by 10¹⁹.
- **128-bit helpers in LLVM IR beside the 64-bit ones**: division a bit
  at a time and `mul` of two unknowns done in line, variable shifts
  built from the 64-bit shift helpers. They return four words in
  `a0`..`a3`, a convention only they use; Rust itself returns a `u128`
  through memory. Rejected: compiler_builtins' `__udivti3` and
  friends, which would pull in much more IR for the same result.

### 2026-11-25 · Floating point from compiler_builtins

Last of the three Rust gaps (`docs/rust.md`, Floating point):

- *Partly superseded the same day: arithmetic, comparisons, sqrt, fma
  and 32/64-bit conversions now run on the binary floating point
  coprocessor (Binary floating point as a coprocessor on the bus,
  below); float.ll kept the libm functions and 128-bit conversions,
  until those were replaced by our own (Our own floating-point
  library, below). No compiler_builtins code is left.*
- **Software floating point, from Rust's own compiler_builtins.**
  The emulator does not run the floating-point instructions isa.md
  specifies, and a ternary FPU is a separate project. rustc already
  lowers `fadd` and friends to `__adddf3` and the rest on a RISC-V
  core without an FPU; doing the same reuses tested IEEE 754 code
  instead of writing it. Rejected: our own soft-float helpers in IR
  (much more code to get right), and floats in balanced ternary (not
  what Rust's `f64` means).
- **The routines live in the repository**, `lower/float.ll`, pruned
  from the IR rustc emits for compiler_builtins and regenerated with
  `lower/float.sh`, so translating a program needs no Rust toolchain
  beyond the one that made its IR. It is third-party code (MIT and
  Apache-2.0, libm from musl), noted in its header.
- **Expanded when the IR is read**, as atomics are: calls, integer
  bit operations for `fneg`/`fabs`/`copysign`, comparisons of the
  `__cmp` results with zero. A `float` is one word of bits, a `double`
  two, so loads, stores, phis and calls need nothing new.
- **The register allocator's coalescing now builds the interference
  graph once** and keeps it up to date as it merges, instead of
  rebuilding it after every merge. The float routines and core's
  float formatting have big functions, and translating the float test
  took 8.7 s; now 0.8 s, with the same output.

### 2026-11-25 · The ternary floating point unit (ISA 0.12)

The emulator now runs isa.md 7.11, the first step of hardware floating
point; binary IEEE 754 for Rust comes next, as a memory-mapped
coprocessor rather than more instructions, so the core stays ternary.

- **Exact, then rounded once.** Every finite float is an integer times
  a power of three, so each operation works out its exact result as
  such a pair (128-bit integers suffice), or keeps enough trits that
  what it drops only decides the direction, and one function rounds.
  The directed modes step one float from the nearest. Checked against
  an exact rational reference on 200,000 random operations.
- **Round to nearest is not quite truncation** (spec change, hence
  0.12). Just above the bottom of an exponent's range the grid below
  is three times finer and its largest value is closer to a value
  that truncates to the bottom significand when that value lies a
  third of a place or more below it. 0.11 said truncation was nearest
  and that only `fdiv` and `fsqrt` could tie; in fact everything can
  tie at that boundary, and `fsqrt` never does. Rejected: keeping
  plain truncation and calling it nearest, which would make the
  format's rounding differ from IEEE's meaning of the word and break
  error bounds at every power of three.
- **NaNs are not propagated**: every NaN result is the default one,
  `E = +364`, `M = 0`, as RISC-V does. There are no signalling NaNs.
- **Underflow is tininess before rounding**, inexact results below the
  smallest normal float.
- **A dynamic rounding mode when `fcsr` holds dynamic or a reserved
  mode is an illegal instruction**, as in RISC-V.
- **`.float` takes decimal literals and rounds them exactly**, with a
  small big-number division, so a constant in a program is the float
  nearest what was written.

### 2026-11-25 · Binary floating point as a coprocessor on the bus

Rust's `f32` and `f64` are IEEE 754 binary floats, which the ternary
FPU cannot stand in for. Eelco chose a memory-mapped coprocessor
(`docs/platform.md`, page −15) over binary instructions in the ISA:

- **The core stays ternary.** Binary floats are foreign to the machine,
  so they live in a device, as x87 once did, and the ISA gains nothing
  binary. Rejected: binary float instructions in the ISA (one
  instruction per operation, but a binary unit in the core), and a
  coprocessor opcode space with its own registers (fast, but more ISA
  for a binary concern).
- **Operands as bit patterns in words**, low and high 32 bits, which is
  how `detrytus ll` already holds floats and 64-bit integers, so
  lowering is a few stores and loads around one OP store. About eight
  instructions per operation, against a hundred or more for the
  software routines.
- **One bank per hart**, so harts need no lock; a kernel saves a
  process's bank like registers.
- **`detrytus ll` uses it** for every operation it has, which
  replaces most of the compiler_builtins calls of the entry "Floating
  point from compiler_builtins"; floats.rs translates to about 29%
  fewer lines. The rest (`floor`, `fmod`, 128-bit conversions) stays
  in software, since in our own code (entry below). The address is the physical one, so an operating
  system maps the page at the same virtual address.
- **Round to nearest even only, no flags, canonical NaNs.** That is all
  Rust's code generation uses. The emulator does the arithmetic with
  the host's IEEE floats, which are correctly rounded in that mode.

### 2026-11-25 · Our own floating-point library

Eelco asked to replace what was left of compiler_builtins in
`lower/float.ll` with our own code:

- **Hand-written LLVM IR, as `lower/wide.ll` is.** About 30 small
  functions: `trunc`, `floor`, `ceil`, `round`, `rint`/`roundeven`,
  `fmod`, `fminimum`/`fmaximum`, and the 128-bit conversions, each
  with a `float` twin. No toolchain, generator script or third-party
  licence is needed. Rejected: our own Rust compiled to IR (needs a
  script and a pinned toolchain to regenerate, and LLVM may turn a
  hand-written `floor` back into `llvm.floor`).
- **Arithmetic on the coprocessor.** Rounding uses its correctly
  rounded adds (2^52 for ties to even), so the functions are short
  and need no rounding logic of their own.
- **`float` versions widen to `double`.** That is exact both ways for
  everything but conversions from 128-bit integers, which would round
  twice; those convert the top 64 bits with a sticky bit at the
  precision wanted and scale by a power of two.
- **Tested against the host** (`rust/tc/tests/float_lib.rs`): edge
  cases and pseudo-random inputs per function, run on the bare
  machine, bit for bit except that any NaN matches a NaN.
- `lower/float.sh` is gone, and `ir.rs` no longer names functions
  nothing reaches (`fma`, `fmin`, `fminimum_num` and kin, which the
  coprocessor does).

### 2026-11-25 · A network device

Eelco asked for a network driver as the first step toward TCP/IP in
trylobite. The machine needed an interface first (`docs/platform.md`,
page −16):

- **Ethernet frames, not packets.** The device moves whole frames and
  knows nothing of IP, as a real NIC, so the whole stack (ARP, IP, TCP)
  is trylobite's to write, which is the point of the exercise. Rejected:
  an IP-level device (a TUN), which would hide ARP and make the frames
  incompatible with every bridge and capture tool.
- **One octet per tryte.** Frames cross to a binary world, so their
  bytes stay bytes, as the coprocessor's values stay bits. A tryte holds
  0..255 with room to spare and software indexes octets directly.
  Rejected: packing octets into trits, which saves memory but makes every
  header field a conversion, for frames that rarely matter in size.
- **Two DMA channels, one buffer each, nine frames waiting in the
  device.** Like the block device: registers, a command, a status, a
  finished state that raises an interrupt until acknowledged. A
  descriptor ring would let a driver queue many buffers, but costs a
  format to specify and a driver to match; the device's nine-frame queue
  covers the bursts a driver will see, and a ring can come later behind
  a new `NET_VERSION`. One interrupt source for both channels, since the
  driver handles both in one place.
- **The send reads the frame at once**, so a failed send says why at
  the command, and the buffer is free straight away. It finishes 100 +
  length rounds later, so a driver has a real wait to fill.
- **The link is looked at every 1,000 rounds**, not every step, so a
  socket costs one system call per thousand steps. While every hart
  waits, time moves to the next look instead of stalling, and with a
  live console those short skips add up to its paced ones, so a kernel
  waiting for packets idles in real time rather than spinning.
- **The outside world over UDP, one datagram per frame.** It needs no
  privileges, works on Linux and macOS, connects two emulators with a
  port each, and is the framing QEMU's `-netdev dgram` and
  `socket,udp=` use, so a QEMU guest (Linux, say) can be the peer that
  answers pings and TCP. A bridge to a host TAP device or a userspace
  NAT can sit at the other end later without changing the emulator.
  Rejected: a TAP device in the emulator (root or a capability on Linux,
  a kernel extension on macOS, and a dependency for the ioctl), and an
  emulator-side NAT like QEMU's slirp (a TCP/IP stack in the emulator,
  which is the thing trylobite is to build). A loopback link keeps runs
  deterministic for tests.

### 2026-11-25 · User-space NAT: `--net user`

trylobite's IP stack wanted the real network. Eelco chose address
translation in user space over a TAP device, which needs root.

- **The emulator plays the gateway**, 10.0.0.1, matching trylobite's
  default address and gateway, so a kernel needs no configuration. The
  link (`rust/cli/src/nat.rs`) reads the machine's frames and turns
  them into host socket calls; nothing in the core changed.
- **TCP is ended in the emulator,** as slirp does: the host side is a
  plain `TcpStream`, and the link speaks TCP to the machine as the far
  host would (handshake after the host's connect succeeds, resets for
  refusals, retransmission on a real-time timer, the machine's window
  respected, one octet into a closed one). Relaying the machine's
  segments raw would need raw sockets and root, the thing avoided.
- **The gateway is the host:** 10.0.0.1 maps to 127.0.0.1, so a test
  can run a server on the host's loopback and reach it from the
  machine, and so can a person.
- **Pings** use an unprivileged ICMP socket where the host allows one
  (Linux's `ping_group_range`, macOS always), else a raw socket for
  root. `std` has neither, so `socket()` is declared by hand, the only
  foreign call in the crate; the descriptor is then an ordinary
  `UdpSocket`. The echo identifier the machine chose is put back on the
  reply, since a ping socket replaces it.
- **No port forwarding into the machine yet,** and no DNS: trylobite
  has no resolver. Either can come when wanted.
- **The connect runs on a thread,** one per connection, so a slow
  connect never stalls the machine; everything else is non-blocking
  sockets polled when the device polls the link.

### 2026-11-25 · A name server on `--net user`

Eelco wanted names on trylobite's network. Like slirp's 10.0.2.3, the
link now plays a name server at 10.0.0.3.

- **Answers come from the host's resolver** (`ToSocketAddrs`), not from
  forwarding the query to a DNS server: that honours the host's hosts
  file and works wherever the host resolves, with no server address to
  find per platform. Each lookup runs on a thread; the answer goes back
  when the link is next polled.
- **A records only.** Any other type gets an empty answer, a failed
  lookup gets NXDOMAIN. The answer echoes the question and uses a
  compression pointer to it, TTL 60.
- **10.0.0.3, not the gateway,** so the name server is a separate
  setting in trylobite, as on a real network.

### 2026-11-26 · Rust's `alloc` crate through `detrytus ll`

Eelco wanted a heap for Rust programs on trylobite. The allocator
itself is the runtime's business (trylobite's, over a kernel call);
what the translator needed was the IR `alloc` brings in:

- **Overflow-reporting arithmetic of up to 32 bits**: `umul` (the
  product as `mul` wraps it, overflow when `b > mask / a`), `uadd` and
  `usub` (already there), `sadd` and `ssub` (the signed sum fits a
  word; overflow when it is out of range). `RawVec` checks its sizes
  with them. `smul` and wider types can come when something needs them.
- **`bitreverse` and `bswap` of up to 32 bits,** a piece at a time
  (alloc's formatting reverses a 3-bit field).
- **The allocator shim's marker is a no-op.** rustc defines
  `__rust_alloc` and the rest in the crate with the
  `#[global_allocator]`, and calls `__rust_no_alloc_shim_is_unstable_v2`,
  which its linker would add, to make sure the shim is linked. Taking
  the call as nothing is what that function does.
- **Constants: a number as a pointer in data** (`inttoptr (i32 1 to
  ptr)`, the dangling pointer of an empty slice in a static), and an
  address plus or minus a number.
- **A runtime's `i32` results are held unsigned.** `bare.s`'s `memcmp`
  returned a negative difference as it was; core's sort tests the sign
  with `lshr 31`, read garbage and indexed off the slice. It now adds
  2^32, as every `i32` is held, and docs/rust.md says so for any
  runtime function.
- **The test is a bump allocator in a static array**
  (`rust/tc/tests/ll/alloc.rs`), which the bare machine needs no heap
  for; `regenerate.sh` builds `alloc` too and prunes it with core.

### 2026-11-26 · RAM of any size, and a register that says how much

Eelco wanted the amount of memory to be arbitrary. The ISA never capped
it: a physical page number is 18 trits, so every positive page up to
193,710,244, the top of the address space, may be RAM. Two things made
it look fixed:

- **Nothing told software the size,** so trylobite built `RAM_PAGES`
  into each kernel and the emulator had to be run with the same count.
  The power page now has a read-only **RAM size** register at −39363,
  next to power off: a fixed platform register, as a real board's
  memory controller or firmware table would say it, rather than having
  the kernel probe for access faults, which would also read device
  pages' side effects into a scan.
- **`--ram-pages` took any number.** It now refuses counts outside
  1..=193,710,244 rather than building a machine whose top pages no
  page number can name.

The emulator's RAM was already sparse (a page is allocated on first
write), so a machine of millions of pages costs only the pages used. A
conformance case maps a root-level superpage to page 3 × 3¹⁶ + 5 to
show the far end of RAM is reachable through paging.

### 2026-11-26 · A faster interpreter, same machine

Eelco asked what could be done about emulator speed. Hardware
virtualisation cannot help: no host runs balanced ternary, so every
instruction is interpreted. A profile of trylobite sorting 10,000
lines (814M steps, 249 s) showed about 3,600 host instructions per
emulated one, most of it bookkeeping rather than the instruction:

- **The trit helpers were not inlined,** so `pow3(n)` ran as a loop and
  every field split was a division by a number the compiler could not
  see. `#[inline]` on `trits.rs` lets the constants fold.
- **SipHash on every access.** Physical pages, the decode caches, the
  MMU's checks and the CSRs are `HashMap`s keyed by integers the
  machine itself makes; no attacker picks them, so collision resistance
  bought nothing. `fastmap.rs` is FxHash's multiply-and-rotate, written
  out because the core has no dependencies.
- **A word read looked its page up three times,** once per tryte. An
  aligned word never straddles a page, so one lookup serves all three.
- **The MMU's map of checks passed** is now a 1,024-entry direct-mapped
  cache that `fence.vm` empties by bumping a generation; a miss falls
  back to the TLB, so a collision costs time, never correctness.

Together: 90 s for the same run, with the same output and step count,
and nothing about the machine changed. The larger steps (a dispatch
loop that runs a hart many instructions at a time, predecoded blocks,
and a JIT) change the core's structure and are planned separately.

### 2026-11-26 · A fast loop between deadlines

The first step towards decoded blocks and a JIT. `Machine::run` runs a
lone running hart in a tight loop for as long as a round's bookkeeping
would change nothing: no device due to finish or be looked at
(`Bus::next_event`), no hot-plug change due, and no access to a device
page since the loop began. Each iteration is still one round, one tick
and one instruction, so step counts and timing are unchanged. Any
access to a device page ends the loop, since it may raise a line,
start a hart or begin a transfer. The binary floating point
coprocessor is the exception: it only computes for the hart using it,
and Rust programs touch it every hundred or so instructions.

`take_interrupt` also checks first whether anything could be pending
(a stored ip trit, the timer due, a line raised) before building ip
trit by trit, which it did on every step. Together: 90 s to 59 s on the
sort benchmark, same output and step count. Most of what is left is
per instruction: fetch, translation and decode, which decoded blocks
remove.

### 2026-11-26 · Decoded code, kept until it is written

Each hart keeps the instructions it has decoded, by physical address,
and the virtual page it last fetched from with how that page
translated. A fetch on that page reads the decoded instruction from an
array: no translation, no bus access, no decode lookup. The page's
translation holds while the ring, `ptbr` and the MMU's generation
(bumped by `fence.vm`) are unchanged, which is exactly as long as the
MMU would give the same answer, since its TLB never evicts on its own.

Code that changes must not run stale. The bus keeps a bit per word that
any hart has decoded; a store, a tryte poke, a block read or a network
delivery over one of those words counts a code write, and every hart
drops its whole cache before its next fetch. The spec only promises a
new instruction after `fence.i`; seeing it at once is stricter and
simpler. Dropping everything is coarse, but code is rarely written
(a program being loaded), and data stores that share a page with code
do not count, being word-exact. Faults are never cached.

Sort benchmark: 59 s to 35 s, same output and step count. This is the
front end a JIT would compile from: a JIT turns these decoded runs into
host code, and drops them by the same rule.

### 2026-11-26 · A JIT on Cranelift

Eelco chose Cranelift over an emitter of our own: he works on Apple
Silicon but did not want Intel left behind, and Cranelift gives arm64
and x86-64 from one code generator, with `cranelift-native` picking up
the CPU's features at run time. The host family itself is fixed when
detrytus is built. To keep the core free of dependencies, the JIT is its
own crate (`rust/jit`) behind a trait the core names
(`accel::Accelerator`); the CLI attaches it, `--no-jit` leaves it off,
and the debugger never uses it. Cranelift needs Rust 1.97, so the CLI
does too; the core alone still builds with 1.88.

**Exactness first.** Compiled code must match the interpreter step for
step, so the machine only offers it rounds inside the fast loop (one
hart, no device due) and no more than `Hart::quiet_steps`, the steps
before the timer could interrupt. Compiled code counts the instructions
it runs and stops at that budget. Loads and stores call back into the
core, which refuses devices (the floating point coprocessor aside) and
anything that would trap; atomics and system instructions are never
compiled. A refusal returns before that instruction and the interpreter
runs it, trap and all. `rust/jit/tests/same.rs` runs the examples, a
loop and self-modifying code both ways and compares output, exit, step
count and registers; the sort benchmark and trylobite's tests match too.

**What is compiled.** Starting from a hot pc, everything reachable
without leaving the page (fall-throughs and branch and jump targets) is
one Cranelift function, so loops run without returning. Code is
compiled after it has been asked for 1,000 times: compiling costs
around a millisecond a function, which short runs, such as the test
suite, would otherwise pay for nothing. Integer arithmetic, compares,
selects and branches are inline; other register-only instructions call
the interpreter's own code for them. Everything compiled is dropped when
a store hits decoded code, by the same rule as the decode cache.

Sort benchmark: 35 s to 22 s, same output and step count. The next gain
is loads and stores inline (a translation cache compiled code can read,
pointing straight at RAM), since every one is a call today.

### 2026-11-26 · Loads and stores in compiled code

Compiled code now loads and stores RAM itself instead of calling back
into the core for every access.

**A page table of its own.** The JIT keeps a small direct-mapped table
from virtual page number to the host memory behind the RAM page
(`Bus::ram_page`, a pointer to the page's centre tryte). Compiled code
splits the address into page and offset (a multiply by a magic
constant, no division), checks the slot's page number, and for a word
that the address is aligned; a hit reads or writes the trytes in place.
A miss calls the core as before, and a successful access then fills the
slot (`Hart::jit_page`), so the table only ever holds translations the
hart's own MMU has just made.

**When a page stops being valid.** A slot is only good while the
hart's translations are unchanged: the same hart, ring, `ptbr`, SUA and
MMU generation (bumped by `fence.vm`). Compiled code runs none of the
instructions that change those. A store slot also needs a page with no
decoded code (else the store must drop decoded code) and no
reservation held by another hart (else the store must break it). All
of this is `MemoryView` (`Hart::jit_view`); the JIT compares it before
each call and keeps tables for the last four views, since a kernel and
its programs alternate. Pages are never freed, so a pointer stays
good as long as the bus.

**More arithmetic inline.** Multiply takes the product modulo 3^27
without 128-bit arithmetic: a quotient estimated in floating point is
within one of the true one, the remainder computed modulo 2^64 with it
is exact, and one correction step finishes it. Divide, remainder and
the binary `band`/`bor`/`bxor` are inline too. `tlut` in the core no
longer divides by a variable power of 3 for each trit, which helps the
interpreter as well; compiled code spreads each `tlut`'s table over
pairs of 3-trit groups once (`alu::TlutGroups`, 729 bytes) and applies
it in 9 lookups instead of 27 (Rust programs use it for every binary
operation).

Sort benchmark: 22 s to about 10.5 s, same output and step count. A
profile now shows the 9% of steps still interpreted (mostly the
kernel's trap entry and exit, `csrrw` and `tret`) as the largest cost;
interpreting one step after such an instruction instead of eight made
no difference.

### 2026-11-26 · The kernel's trap path in compiled code

CSR accesses, `tret` and the atomics (the kernel lock's `lr`, `sc` and
`amoswap`) are compiled now instead of ending a block, which sent the
next eight steps to the interpreter. They are a class of their own,
`Class::System`: compiled code calls `Hart::jit_system`, which runs the
interpreter's own code for the instruction and then ends the run, as
such an instruction may change what bounded it (interrupt enables, the
timer, the ring, `ptbr`, reservations).

Two things keep it exact. A refusal must change nothing, so
`jit_system` first checks every way the instruction could trap (a CSR
the ring may not read or may not write, a misaligned or untranslatable
address, anything but RAM) and only then runs it; reading `entropy`
advances its stream, so the read must not happen before a write that
traps. And the clock and counters a CSR reads are the ones the machine
would have set for that step: compiled code passes how many
instructions ran before it in the run, and `time`, `cycle` and
`instret` are adjusted for the one read.

Steps still interpreted on the sort benchmark: 9% to under 4%, mostly
trap entry now (`sys` and interrupts). Sort benchmark: about 10.5 s to
about 9.3 s, same output and step count.

### 2026-11-26 · Signed pointers in `detrytus ll`, behind a switch

A Rust pointer was an unsigned 32-bit number, so nothing below address
0 could be one, and a kernel lives there (trylobite's code runs near
−68 900, its device pages from −9 to −1 pages). With
`--signed-pointers`, pointers are signed 32-bit addresses: the address
in a register, its two's complement in memory and in integers,
converted where a pointer is loaded, stored or cast (docs/rust.md,
Pointers). It is the first step of porting trylobite's kernel to Rust.

Two other ways were weighed. Moving the kernel and its direct map
above 0 and below 2³² keeps `ll` as it was but abandons the balanced
layout. Pointers a whole word wide reach every address but make
`usize` wider than 32 bits, which every Rust program would pay for.

It is a switch, not the default, because it costs: trylobite's sort
benchmark, which loads pointers all the time, took 12.6% more steps
with the conversions inline, and still 10% (810 to 892 million) with
them inside pointer-only load and store helpers and a branch instead
of a wrap. A program running above 0 gains nothing for that, so only
the kernel pays it; without the switch the output is unchanged
(trylobite's disk image is byte for byte the same).
