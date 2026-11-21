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
