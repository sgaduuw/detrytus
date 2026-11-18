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

These were proposed in the pull request that implemented paging, for
the owner to accept or change.

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

