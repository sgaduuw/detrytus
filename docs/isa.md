# detrytus ISA

Version 0.6, **draft**. Everything here may change until the emulator
has run real programs against it. Sections marked *sketch* record a
direction, not a rule.

detrytus is a balanced-ternary instruction set in the spirit of the
Soviet Setun (1958) and Setun-70 (1970), rebuilt around what a modern
core offers an operating system: a load/store RISC design with
position-independent code, three privilege rings with trap delegation,
paged virtual memory, a timer, any number of harts up to the platform's
limit (hot-pluggable), a weak memory model with acquire/release atomics,
floating point, cache control and an entropy source. Where
ternary lets a feature be simpler than its binary equivalent, the spec
takes the simpler form and says so.

It is designed to host its own operating system. It is not designed to
run C-era binary software such as Linux, whose language and kernel
assume binary integers and 8-bit bytes throughout.

## 1. Notation

A **trit** takes one of three values: −1, 0, +1, written `-`, `0`, `+`.

A sequence of trits is written most significant first. In assembly a
balanced-ternary literal carries the prefix `0t`, the way `0b` marks
binary: `0t+0-` is 9 − 1 = 8. Decimal literals are also accepted and
are the default in this document.

Trit positions are numbered from 0 at the least significant end. `x[i]`
is trit `i` of `x`; `x[j:i]` is the field from trit `j` down to trit `i`
inclusive. A field is read as a balanced number in its own right, so a
field's value can be negative.

`wrap(x)` reduces an integer into the 27-trit word range by arithmetic
modulo 3²⁷, choosing the representative in
[−3,812,798,742,493, +3,812,798,742,493].

`Hn` is `(3ⁿ − 1) / 2`, the largest value an `n`-trit field holds.

## 2. Why balanced ternary changes the design

The rest of the spec leans on these properties.

- **No sign bit, no unsigned types.** The sign of a number is the sign
  of its most significant nonzero trit. There is exactly one kind of
  integer: no unsigned compares, no unsigned multiplies, no zero-extend
  versus sign-extend loads.
- **Negation is tritwise inversion.** Flipping every trit negates the
  value, so `not` and `neg` are the same operation and the range is
  symmetric: there is no most-negative number that cannot be negated,
  and no negative zero in either integers or floats.
- **Truncation rounds to nearest.** Dropping low trits leaves the
  nearest integer to the exact quotient. A right shift is therefore a
  correctly rounded division by a power of three.
- **Comparison never overflows.** Ordering is lexicographic on trits,
  most significant first, with `-` < `0` < `+`. A compare needs no
  subtraction.
- **A compare has three outcomes**, and the outcome is itself a value:
  it can be stored, branched on three ways, or used to select.
- **Fields split without carries.** Any integer is uniquely
  `hi × 3ⁿ + lo` with `lo` in the n-trit balanced range. Constants build
  from an upper and a lower immediate with no rounding fix-up, and an
  address splits into page number and offset just by cutting its trits.
- **Everything is centred.** Because fields are signed, a page, a word
  or a table is naturally centred on its base address rather than
  starting at it (section 5). This looks odd at first and removes every
  offset adjustment from address arithmetic.

The costs are real. Halving is not a shift, so algorithms built on
powers of two (binary search midpoints, buddy allocators, hash masks)
pay for a division or are redesigned around thirds. And everything
outside the core speaks binary: section 11 covers how detrytus talks to
it.

## 3. Data types

| Unit  | Trits | Range                         | Approx. binary equivalent |
|-------|-------|-------------------------------|---------------------------|
| trit  | 1     | −1 .. +1                      | 1.58 bits                 |
| tryte | 9     | ±9,841                        | 14.3 bits                 |
| word  | 27    | ±3,812,798,742,493            | 42.8 bits                 |
| float | 27    | about 10^±173, 9.5 digits      | between binary32 and binary64 |

The word is three trytes. Keeping every size a power of three keeps
every field boundary clean. A 27-trit address space is about 7.6 × 10¹²
trytes, roughly 13 TB of binary-equivalent storage, which is ample for
this project; open question 1 records the alternative.

## 4. Programmer-visible state

### 4.1 Registers

There are 27 general-purpose registers, each one word, named by a
balanced 3-trit index from −13 to +13: `r-13` .. `r-1`, `r0`, `r1` ..
`r13`. `r0` always reads as zero and ignores writes.

Integers and floats share this register file. There is no separate
floating-point register file, which keeps context switches small: a
microkernel saves 26 words and `pc`, nothing more.

The program counter `pc` is a separate word register. It holds a
multiple of 3, or one more than a multiple of 3 when it points at the
second slot of a compressed bundle (6.1).

### 4.2 Calling convention

| Register       | ABI name     | Role                         | Saved by |
|----------------|--------------|------------------------------|----------|
| `r0`           | `zero`       | constant zero                |          |
| `r1`..`r6`     | `a0`..`a5`   | arguments; `a0` returns      | caller   |
| `r7`..`r13`    | `t0`..`t6`   | temporaries                  | caller   |
| `r-1`..`r-9`   | `s0`..`s8`   | saved; `s0` is the frame pointer | callee |
| `r-10`         | `gp`         | global pointer               |          |
| `r-11`         | `tp`         | thread pointer (TLS)         |          |
| `r-12`         | `ra`         | return address               | caller   |
| `r-13`         | `sp`         | stack pointer                | callee   |

Positive registers are scratch; negative ones survive a call. The stack
grows toward negative addresses. A boolean is a trit: `+` true, `-`
false, and `0` where a three-valued result needs "unknown".

## 5. Memory

### 5.1 Addressing

Memory is tryte-addressable. An address is a word, so addresses are
signed and run from −H27 to +H27, centred on 0. Address arithmetic wraps
like all integer arithmetic, so the address space is a ring of 3²⁷
trytes: one step past +H27 is −H27, and no address is the first or the
last.

### 5.2 Centred layout

An aligned unit of 3ᵏ trytes is centred on its address. Its address
must be a multiple of 3ᵏ, which in balanced ternary is exactly "the low
`k` trits of the address are 0", and the unit covers the address and
H`k` trytes on each side.

| Unit        | Trytes | Aligned to | Covers, for address `a`      |
|-------------|--------|------------|------------------------------|
| word        | 3      | 3          | `a−1` .. `a+1`               |
| cache line  | 27     | 27         | `a−13` .. `a+13`             |
| page        | 19,683 | 3⁹         | `a−9841` .. `a+9841`         |

This is what splitting balanced trits gives without help: the low `k`
trits of any address are a signed offset from the nearest multiple of
3ᵏ. Under a start-based layout a page centred on its offsets would need
an adjustment on every translation, and the last word of a page would
straddle the next one.

Centring and the ring fit together. Because 3²⁷ is a multiple of every
unit size, centred units tile the ring exactly and none of them
straddles the seam between +H27 and −H27. Under a start-based layout the
word at the highest aligned address, H27 − 1, would cover H27 − 1 and
H27 and then wrap round to −H27.

Centring decides which trytes make up a unit; byte order decides their
order inside it. A word is stored low trits first: tryte `a−1` holds
trits 8..0, `a` holds 17..9, and `a+1` holds 26..18. Significance grows
with address, the ternary form of little-endian.

A misaligned word access traps (cause +2). Branch targets are always
aligned words, and branch offsets count words.

### 5.3 Physical address map

The sign of the physical page number (the physical address with its
low 9 trits cut off) selects one of three regions:

| Page number | Region |
|-------------|--------|
| positive    | RAM: cacheable, weakly ordered |
| zero        | boot page: reset firmware, readable and executable |
| negative    | MMIO: uncached, strongly ordered, no execution |

Deciding whether an access is memory or a device is a single trit test,
with no attribute table. Accessing an address that holds nothing traps
(cause +3).

### 5.4 Reset

On reset hart 0 starts with `pc` 0, inside the boot page. All registers
are 0, the current ring is +1, interrupts are disabled and translation
is off. The all-zero word is an illegal instruction (opcode 0), so
execution that runs into zeroed memory traps instead of sliding.

Every other hart present at reset starts *stopped*: it runs nothing
until a running hart starts it through the platform's hart controller
(5.6), at an address and with an argument of that hart's choosing, in
the same state as hart 0 at reset. Hart 0 is always present. Firmware
therefore needs no code to park secondary harts, and a hart plugged in
later behaves exactly like one that was present but not yet started.

### 5.5 Memory model

The memory model is weak, modelled on RISC-V's RVWMO and simplified
where RVWMO's extra freedom buys little: loads from the same address are
always ordered, and only three plain kinds of dependency carry order.

**Definitions.** A memory operation is a load, a store, or both at once
(an AMO, or an `sc` that succeeds), on a set of trytes. Program order is
the order of one hart's operations. An execution is legal when a
**global memory order** exists: one total order over every memory
operation of every hart, satisfying the rules below.

**Preserved program order.** If `a` comes before `b` in program order,
`a` also comes before `b` in the global memory order when any of these
holds:

1. **Same address:** `a` and `b` share a tryte, and `b` is a store, or
   both are loads.
2. **Fence:** a `fence` lies between them, with `a` in its `pred` set
   and `b` in its `succ` set.
3. **Acquire:** `a` carries `aq`.
4. **Release:** `b` carries `rl`.
5. **Release then acquire:** `a` carries `rl` and `b` carries `aq`, which
   makes annotated operations sequentially consistent with each other.
6. **Reservation:** `a` is an `lr` and `b` is the `sc` paired with it.
7. **Dependency:** `b`'s address is computed from the value `a`
   loaded; or `b` is a store whose data is computed from it; or `b` is
   a store after a branch whose condition is computed from it.
8. **Device order:** `a` and `b` both access MMIO.

**Axioms.**

- **Load value.** A load of a tryte returns the value written by the
  latest store to that tryte, taking the latest in global memory order
  among the stores that precede the load in global memory order or in
  its own hart's program order. A hart therefore sees its own stores at
  once, and other harts see a store at a single point. With no such
  store the value is 0.
- **Atomicity.** An AMO is a single operation. For an `sc` that
  succeeds, no store from another hart to the reserved word falls
  between the `lr` and the `sc` in global memory order.
- **Progress.** No store is delayed forever: each one precedes all but
  finitely many operations in global memory order.

Aligned words and single trytes are single-copy atomic. Instruction
fetch and page-table walks are separate observers: they are guaranteed
to see earlier stores only after `fence.i` and `fence.vm` respectively.

Sequential consistency is one legal outcome, so an emulator that runs
harts one instruction at a time with no store buffering conforms
without further work.

### 5.6 Multiple harts

A machine has one or more **harts** (hardware threads), each with its
own `pc`, registers, CSRs (both trap banks included), current ring, TLB
and `lr` reservation. They share everything else: all of physical
memory, which is coherent between them under the memory model above;
the devices; and one clock, the `time` CSR (8.2), which reads the same
on every hart. Each hart compares it against its own `timecmp`
registers, so each has its own timer interrupt.

**Numbering.** A hart's number, read from `hartid`, is balanced and
names its place in a tree of threes: the lowest trit is its position
within a triad (−1, 0, +1), the next trit the triad's position within a
group of nine, and so on up. A machine of 9 harts numbers them −4 to +4;
one of 27, −13 to +13. Hart 0, the middle of the middle triad at every
level, is the boot hart. Numbers are fixed slots, so a machine need not
fill them: with hot-plug, gaps are normal, and software must not assume
that harts −n..n all exist. Hardware may share caches within a triad
and connect triads as a ternary tree; the numbering follows such a
layout naturally, but the ISA does not require one. The platform sets
the largest number of harts (the reference platform allows 729, hart
numbers −364..+364).

**Starting, stopping, hot-plug.** A hart is *absent*, *stopped* or
*running*. A running hart starts a stopped one, at an entry address and
with one argument, through the platform's hart controller
(`platform.md`); the new hart begins in the reset state of 5.4 with
`a0` holding the argument. A hart stops only itself, which makes
removal cooperative: the kernel moves work off a hart, then the hart
stops, and only a stopped hart may be removed. A stopped hart holds no
state: its registers, TLB, reservation and pending messages are gone.
Harts may be added or removed while the machine runs; the controller
announces either with an external interrupt.

**Per-hart storage.** There is no hart-private memory: one address
means the same location on every hart. Software keeps per-hart data (a
stack, a trap save area, scheduler state) in ordinary RAM and reaches it
through `hartid`, the `scratch` CSRs (a trap handler swaps its area's
address in with `csrrw`), the `tp` register by convention, or the
argument it was started with.

**Signals and messages.** Each hart has a small mailbox in the hart
controller. One store to another hart's mailbox queues a word for it and
raises its software interrupt (cause −3), which stays pending while the
mailbox holds messages. A sender can address one hart, or every running
hart in a subtree of the numbering (all harts that agree with a number
except in its lowest `k` trits). Larger payloads go in shared RAM, with
the message saying where; the sender orders the two with `fence w, o`
between filling the buffer and sending, and the receiver with
`fence i, r` between taking the message and reading the buffer.

**Remote flushes.** `fence.vm` and `fence.i` act on the hart that runs
them. A kernel that changes a translation other harts may hold, or code
they may run, signals them and has each run the fence itself: the
usual shootdown. A stopped hart holds nothing to flush.

## 6. Instruction encoding

Every instruction is one 27-trit word. The opcode sits in the low five
trits, giving 243 major opcodes, numbered −121..+121. Opcode 0 is
permanently illegal. Each instruction has its own major opcode, so
decode is a single table lookup with no secondary function field.

Register fields keep the same position in every format that has them,
so the register file can be read before the format is known.

```
trit:  26 ........... 17 | 16 14 | 13 11 | 10  8 | 7   5 | 4   0
R      fn[10]            | rs3   | rs2   | rs1   | rd    | op
I      imm[16] ......................... | rs1   | rd    | op
S      imm[16] ......................... | rs1   | rs2   | op
B      off[13] ............... | rs2     | rs1   | h c c | op
T      offP[8] ..... | offN[8] .....     | rs1   | 0 0 h | op
U      imm[19] ................................  | rd    | op
J      off[19] ................................  | rd    | op
```

| Format | Fields |
|--------|--------|
| R | `rd`, `rs1`, `rs2`, `rs3`, `fn` (10 trits) |
| I | `rd`, `rs1`, `imm` (16 trits, ±21,523,360) |
| S | `rs2` (in the `rd` slot), `rs1`, `imm` (16 trits) |
| B | `cond` (trits 6..5), hint `h` (trit 7), `rs1`, `rs2`, `off` (13 trits, ±797,161 instructions) |
| T | hint `h` (trit 5), trits 7..6 zero, `rs1`, `offN`, `offP` (8 trits each, ±3,280 instructions) |
| U | `rd`, `imm` (19 trits) |
| J | `rd`, `off` (19 trits, ±581,130,733 instructions) |

Unused register fields must name `r0`, and unused `fn` or `imm` trits
must be 0. Anything else is an illegal instruction, which keeps that
encoding space free for later extensions.

Branch and jump targets are `pc + 3 × off`, relative to the address of
the branch itself.

### 6.1 Compressed bundles

A word whose opcode is +121 (`0t+++++`) is a **bundle**: two 11-trit
compressed instructions in place of one full instruction. Slot A is
trits 15..5 and runs first; slot B is trits 26..16. Every compressed
instruction expands to exactly one full instruction, so bundles add
density, not behaviour, and a decoder expands them before execution.

A compressed register field is 3 trits, so it reaches all 27 registers.
RISC-V's compressed forms reach only 8.

**Addressing.** Slot A's `pc` is the bundle's address `a`; slot B's is
`a + 1`. This uses the low trit of `pc`, which full instructions leave
at 0: `+` names slot B, and `-` is reserved. After slot B, execution
continues at `a + 3`. A taken branch in slot A skips slot B. Branch and
jump targets are always word addresses; slot B is entered only by
falling through, by `jalr` to a saved return address, or by `tret` after
a trap in slot B. An unused slot holds `c.nop`. Compressed branch
offsets are relative to the bundle's address, whichever slot holds them.

```
trit:  10 9 | 8   6 | 5   3 | 2   0
CR     f[2] | rs    | rd    | op
CI     imm[5]       | rd    | op       imm: ±121
CJ     off[8]               | op       off: ±3,280 words
```

| Op | Format | Compressed | Expands to |
|----|--------|-----------|------------|
| 0  |    | (illegal; an all-zero slot traps) | |
| 1  | CR | `c.mv` (f 0), `c.add` (+1), `c.sub` (−1), `c.cmp` (+2), `c.mul` (−2), `c.min` (+3), `c.max` (−3), `c.tmin` (+4), `c.tmax` (−4) `rd, rs` | `addi rd, rs, 0`; otherwise the op as `rd = rd op rs` |
| 2  | CR | `c.lw` (f 0), `c.sw` (+1), `c.lt` (−1), `c.st` (+2) `rd, (rs)` | `lw rd, 0(rs)` etc.; for stores `rd` is the source |
| 3  | CR | `c.jr rs` (f 0), `c.jalr rs` (+1) | `jalr r0, 0(rs)`; `jalr ra, 0(rs)` |
| 4  | CI | `c.addi rd, imm` | `addi rd, rd, imm`; with `rd = r0` and `imm = 0` this is `c.nop` |
| 5  | CI | `c.li rd, imm` | `addi rd, r0, imm` |
| 6  | CI | `c.shi rd, imm` | `shi rd, rd, imm` |
| 7  | CI | `c.cmpi rd, imm` | `cmpi rd, rd, imm` |
| 8  | CI | `c.lwsp rd, imm` | `lw rd, 3·imm(sp)` |
| 9  | CI | `c.swsp rd, imm` | `sw rd, 3·imm(sp)` |
| 10 | CI | `c.beqz rd, imm` | `beq rd, r0, imm` |
| 11 | CI | `c.bnez rd, imm` | `bne rd, r0, imm` |
| 12 | CJ | `c.j off` | `jal r0, off` |

`c.jr ra` is `ret`. Unlisted opcodes and `f` values are reserved.
Which instructions deserve a slot should be revisited once the
emulator can count instruction frequencies in real code.

## 7. Instructions

There is one profile: every instruction in this section is mandatory.
Optional extensions fragment software, and a single-implementation
project gains nothing from them. The vector extension (12.1) is the
one planned exception.

All integer results pass through `wrap`.

### 7.1 Integer arithmetic (R)

| Op | Mnemonic | Semantics |
|----|----------|-----------|
| 1  | `add rd, rs1, rs2`  | `rs1 + rs2` |
| 2  | `sub rd, rs1, rs2`  | `rs1 − rs2` |
| 3  | `mul rd, rs1, rs2`  | low word of the 54-trit product |
| 4  | `mulh rd, rs1, rs2` | high word of the 54-trit product |
| 5  | `div rd, rs1, rs2`  | quotient truncated toward zero |
| 6  | `rem rd, rs1, rs2`  | remainder of `div`, with the sign of `rs1` |
| 7  | `divr rd, rs1, rs2` | quotient rounded to nearest, ties toward zero |
| 8  | `remr rd, rs1, rs2` | remainder of `divr`, at most half of `rs2` in magnitude |

The 54-trit product splits uniquely as `hi × 3²⁷ + lo`, so one `mulh`
does what RISC-V needs `mulh`, `mulhu` and `mulhsu` for.

`div`/`rem` give C's familiar semantics. `divr`/`remr` are the
ternary-native pair: the remainder lands in the balanced range, the
way a right shift does. Division cannot overflow, since the range is
symmetric. Division by zero does not trap: the quotient is 0 and the
remainder is `rs1`, which keeps `rs1 = q × rs2 + r` true for both pairs.

### 7.2 Compare, select, min/max (R)

| Op | Mnemonic | Semantics |
|----|----------|-----------|
| 9  | `cmp rd, rs1, rs2`      | `sign(rs1 − rs2)`, computed without overflow |
| 10 | `min rd, rs1, rs2`      | numeric minimum |
| 11 | `max rd, rs1, rs2`      | numeric maximum |
| 12 | `abs rd, rs1`           | magnitude (`rs2` must be `r0`) |
| 13 | `sel rd, rs1, rs2, rs3` | `rs2` if `rs1 < 0`, `0` if `rs1 = 0`, `rs3` if `rs1 > 0` |

With booleans as trits, `sel` is an ordinary select (false picks `rs2`,
true picks `rs3`), and an unknown condition yields 0. It reads three
registers, never four.

### 7.3 Shifts and trit fields (R, I)

| Op | Mnemonic | Format | Semantics |
|----|----------|--------|-----------|
| 14 | `sh rd, rs1, rs2`    | R | shift left by `rs2` trits; a negative `rs2` shifts right |
| 15 | `rot rd, rs1, rs2`   | R | rotate left by `rs2` trits, taken modulo 27 |
| 30 | `shi rd, rs1, imm`   | I | `sh` by an immediate |
| 31 | `roti rd, rs1, imm`  | I | `rot` by an immediate |
| 32 | `tex rd, rs1, pos, len` | I | `rd = rs1[pos+len−1 : pos]`, as a balanced value |
| 33 | `tins rd, rs1, pos, len` | I | `rd[pos+len−1 : pos] = rs1[len−1 : 0]`; other trits of `rd` keep their value |

A left shift by `n` is `wrap(rs1 × 3ⁿ)`. A right shift by `n` drops the
low `n` trits, which is `rs1 / 3ⁿ` rounded to nearest; ties cannot occur
because 3ⁿ is odd. Shifts of 27 or more trits give 0. One signed shift
replaces the usual three (left, logical right, arithmetic right).

For `tex` and `tins`, `pos` is `imm[3:0]` and `len` is `imm[7:4]`, with
`0 ≤ pos`, `1 ≤ len` and `pos + len ≤ 27`. They are the tools for
packing fields such as page-table entries.

### 7.4 Trit counting (R)

All take `rs2 = r0`.

| Op | Mnemonic | Semantics |
|----|----------|-----------|
| 16 | `lzt rd, rs1` | number of leading zero trits (27 for zero) |
| 17 | `tzt rd, rs1` | number of trailing zero trits (27 for zero): the power of 3 dividing `rs1` |
| 18 | `nzt rd, rs1` | number of nonzero trits |

These are the ternary counterparts of `clz`, `ctz` and `popcount`.

### 7.5 Tritwise logic (R)

| Op | Mnemonic | Semantics |
|----|----------|-----------|
| 19 | `tlut rd, rs1, rs2, table` | for each trit `i`: `rd[i] = table[3·(rs1[i]+1) + (rs2[i]+1)]` |

`table` is a 9-trit truth table held in `fn[8:0]` (`fn[9]` must be 0).
Entry `k` is the output for the input pair
`(a, b) = (k div 3 − 1, k mod 3 − 1)`, so entry 0 is `(-,-)` and entry 8
is `(+,+)`. One instruction expresses all 19,683 two-input trit
functions, the way AVX-512's `vpternlog` covers every three-input
boolean function.

The assembler names the common tables:

| Pseudo  | Function | Table (decimal) | Table (`0t`, entry 8 first) |
|---------|----------|-----------------|------------------------------|
| `tmin`  | Kleene AND, `min(a, b)`           | 5792  | `0t+0-00----` |
| `tmax`  | Kleene OR, `max(a, b)`            | 9728  | `0t++++00+0-` |
| `tmul`  | `a × b` (the ternary XNOR)        | 5824  | `0t+0-000-0+` |
| `tsum`  | `a + b` modulo 3, no carry        | −4160 | `0t-+0+0-0-+` |
| `tcons` | consensus: `a` if `a = b`, else 0 | 6560  | `0t+0000000-` |
| `tany`  | accept-anything: `sign(a + b)`    | 8960  | `0t++0+0-0--` |

Unary functions use `rs2 = r0`, so only entries 1, 4 and 7 matter. The
pseudo `tmap rd, rs, f` takes any of the 27 unary trit functions; for
example `tabs` (9490) maps every trit to its magnitude. Negation is
arithmetic `neg`.

Kleene logic gives a natural "unknown". A word can carry
true/false/unknown in each of its 27 trits, the shape of SQL `NULL`
semantics or an initialised/uninitialised shadow map, and `tmin`/`tmax`
propagate the unknowns correctly.

### 7.6 Binary operations (R)

| Op | Mnemonic | Semantics |
|----|----------|-----------|
| 20 | `band rd, rs1, rs2` | bitwise AND of the two's-complement forms |
| 21 | `bor rd, rs1, rs2`  | bitwise OR |
| 22 | `bxor rd, rs1, rs2` | bitwise XOR |
| 23 | `bsh rd, rs1, rs2`  | shift left by `rs2` bits; negative shifts right, rounding toward −∞ |

These treat each operand as an integer in infinite-precision two's
complement and wrap the result. They exist for the binary world:
checksums, CRCs, hash functions, network headers and cryptography
specified in bits. Ternary code has no need of them.

### 7.7 Immediates and PC-relative addressing (I, U)

| Op | Mnemonic | Format | Semantics |
|----|----------|--------|-----------|
| 34 | `addi rd, rs1, imm` | I | `rs1 + imm` |
| 35 | `cmpi rd, rs1, imm` | I | `sign(rs1 − imm)` |
| 36 | `lui rd, imm`       | U | `imm × 3⁸` |
| 37 | `auipc rd, imm`     | U | `pc + imm × 3⁸` |

`lui` + `addi` builds any constant, and `auipc` + `addi` or `auipc` + a
load reaches any address relative to `pc`, with the low 8 trits in the
second instruction and no carry correction (section 2). `auipc` is what
makes position-independent code and relocatable kernels practical.

### 7.8 Loads and stores (I, S)

| Op | Mnemonic | Format | Semantics |
|----|----------|--------|-----------|
| −1 | `lt rd, imm(rs1)`  | I | load the tryte at `rs1 + imm`; upper trits are 0 |
| −2 | `lw rd, imm(rs1)`  | I | load the aligned word at `rs1 + imm` |
| −3 | `st rs2, imm(rs1)` | S | store trits 8..0 of `rs2` to the tryte at `rs1 + imm` |
| −4 | `sw rs2, imm(rs1)` | S | store `rs2` to the aligned word at `rs1 + imm` |

Filling with zero trits preserves the value, so one tryte load does the
work of a signed and an unsigned pair.

### 7.9 Atomics (R)

The address is in `rs1` with no offset and must be word aligned. The
`rs3` field carries the ordering: trit 14 is acquire (`aq`), trit 15 is
release (`rl`), each `0` or `+`, and trit 16 is 0. Both together make
the operation sequentially consistent. Assembler suffixes: `.aq`, `.rl`,
`.aqrl`.

| Op  | Mnemonic | Semantics |
|-----|----------|-----------|
| −20 | `lr rd, (rs1)`            | load and reserve the word |
| −21 | `sc rd, rs2, (rs1)`       | store if the reservation holds; `rd = 0` on success, `+1` on failure |
| −22 | `amoswap rd, rs2, (rs1)`  | `rd = mem`; `mem = rs2` |
| −23 | `amoadd rd, rs2, (rs1)`   | `rd = mem`; `mem = rd + rs2` |
| −24 | `amomin rd, rs2, (rs1)`   | `rd = mem`; `mem = min(rd, rs2)` |
| −25 | `amomax rd, rs2, (rs1)`   | `rd = mem`; `mem = max(rd, rs2)` |
| −26 | `amotlut rd, rs2, (rs1), table` | `rd = mem`; `mem = tlut(rd, rs2, table)`, table in `fn[8:0]` |
| −27 | `cas rd, rs2, (rs1)`      | if `mem = rd` then `mem = rs2`; `rd = ` the old `mem` |

`amotlut` is one atomic instruction for every tritwise update: with
`tmax` it sets trits, with `tmin` it clears them, with `tsum` it cycles
them, which covers what RISC-V spreads across `amoand`, `amoor` and
`amoxor`. Every integer is signed, so `amomin`/`amomax` need no
unsigned variants. `cas` reads its expected value from `rd` (as x86's
`cmpxchg` does) so that it fits a three-register format.

**Proposed: waiting on a reservation.** *Under review; not yet in the
opcode table above, and opcode −28 is held for it.* `wrs rs1` (format I,
`rd` and `imm` zero) lets a hart sleep until another hart may have
changed a word it is watching, instead of spinning on loads. The word is
the one its last `lr` reserved. `wrs` returns, and execution goes on
with the next instruction, at the latest when one of these holds:

1. The hart holds no reservation: it never took one, or lost it, for
   instance to another hart's store to the reserved word (5.5).
2. An interrupt is pending and enabled in `ie` in either bank, whatever
   the current ring and `IE` bits: what ends a `wfi`.
3. `rs1` names a register other than `r0`, and `time ≥ rs1`: a
   deadline, so that a waiter can give up and do something else.

It may also return early for no reason, so it belongs in a loop that
checks the word again; a hart that cannot wait may treat it as a no-op,
as it may `wfi`. `wrs` keeps the reservation, so an `sc` after it
succeeds if nothing broke the reservation meanwhile. It is not a memory
operation and orders nothing. It is legal in every ring: a user program
cannot keep a hart asleep, since the kernel's timer interrupt, enabled
in its `ie`, ends the wait.

Waiting for a lock to come free:

```
acquire: lr    t1, (a0)        # reserve the lock word
         beqz  t1, take        # free: try to take it
         wrs   zero            # held: sleep until someone writes it
         j     acquire
take:    li    t2, 1
         sc.aq t1, t2, (a0)    # 0 if no one got there first
         bnez  t1, acquire
```

No wake-up is lost: if the holder releases the lock between the `lr`
and the `wrs`, its store has already broken the reservation, and `wrs`
returns at once.

### 7.10 Control transfer (B, T, J, I)

| Op  | Mnemonic | Format | Semantics |
|-----|----------|--------|-----------|
| −10 | `jal rd, label`           | J | `rd = pc + 3`; `pc = pc + 3 × off` |
| −11 | `jalr rd, imm(rs1)`       | I | `rd = pc + 3`; `pc = rs1 + imm`, which must be aligned or name a slot B |
| −12 | `b<cond> rs1, rs2, label` | B | if `cmp(rs1, rs2)` satisfies `cond`, `pc = pc + 3 × off` |
| −13 | `br3 rs1, Lneg, Lpos`     | T | `rs1 < 0`: `pc += 3 × offN`; `rs1 > 0`: `pc += 3 × offP`; `rs1 = 0`: fall through |

The `cond` field of `b<cond>`:

| `cond` | Mnemonic | Taken when `cmp(rs1, rs2)` is |
|--------|----------|-------------------------------|
| 0      | `beq`    | 0      |
| +3     | `bne`    | −1, +1 |
| +1     | `bgt`    | +1     |
| −1     | `blt`    | −1     |
| +2     | `bge`    | 0, +1  |
| −2     | `ble`    | −1, 0  |
| −3, ±4 | reserved |        |

Negating an ordered `cond` mirrors the comparison, which is the same as
swapping the operands.

**Static prediction hints.** Formats B and T have a spare trit, used as
a hint at no cost. In `b<cond>` it is −1 for unlikely, 0 for no hint,
+1 for likely, written `.unlikely` / `.likely`. In `br3` it is the
predicted sign of `rs1`, written `.n` / `.z` / `.p`. Hints never change
behaviour.

`br3` is the instruction Setun's design points at: one compare, one
branch, three destinations. Its offsets are short; a far target goes
through a `jal` trampoline.

### 7.11 Floating point (R)

| Op | Mnemonic | Semantics |
|----|----------|-----------|
| 40 | `fadd rd, rs1, rs2`       | `rs1 + rs2` |
| 41 | `fsub rd, rs1, rs2`       | `rs1 − rs2` |
| 42 | `fmul rd, rs1, rs2`       | `rs1 × rs2` |
| 43 | `fdiv rd, rs1, rs2`       | `rs1 / rs2` |
| 44 | `fsqrt rd, rs1`           | `√rs1` |
| 45 | `fma rd, rs1, rs2, rs3`   | `rs1 × rs2 + rs3`, rounded once |
| 46 | `fmin rd, rs1, rs2`       | minimum; a NaN operand is ignored |
| 47 | `fmax rd, rs1, rs2`       | maximum; a NaN operand is ignored |
| 48 | `fcmp rd, rs1, rs2`       | three-way compare in total order, NaNs included |
| 49 | `fclass rd, rs1`          | class code (below) |
| 50 | `fcvt.f.i rd, rs1`        | integer to float |
| 51 | `fcvt.i.f rd, rs1`        | float to integer; out of range saturates, NaN gives 0 |
| 52 | `feq rd, rs1, rs2`        | `rs1 = rs2`: `+` true, `-` false, `0` if unordered |
| 53 | `flt rd, rs1, rs2`        | `rs1 < rs2`: `+` true, `-` false, `0` if unordered |
| 54 | `fle rd, rs1, rs2`        | `rs1 ≤ rs2`: `+` true, `-` false, `0` if unordered |

Arithmetic and conversion instructions take a rounding mode in
`fn[1:0]`: 0 nearest, +1 toward +∞, −1 toward −∞, +3 toward zero, −3
dynamic (from `fcsr`), others reserved. The common case, nearest, is the
all-zero field.

**Format.** A float is one word: exponent `E = x[26:21]` (6 trits) and
significand `M = x[20:0]` (21 trits).

| `E` | Meaning |
|-----|---------|
| −363 .. +363 | normal: value = `M / 3²⁰ × 3^E`, with `M[20] ≠ 0`, or zero when `M = 0` |
| −364 | subnormal: value = `M / 3²⁰ × 3^−363`, `M[20]` may be 0 |
| +364 | `M = +3²⁰`: +∞; `M = −3²⁰`: −∞; any other `M`: NaN |

The sign is the sign of the significand, so there is no sign bit and no
negative zero. The all-zero word is integer 0 and float 0.0 at once.
Negation is `fsub rd, zero, rs`. A normalised significand has magnitude
between ½ and 1½, so the format holds about 9.5 decimal digits over a
range of about 10^±173.

**Rounding.** Round-to-nearest is truncation of the significand. A tie
needs a remainder of exactly half a unit in the last place, which has
an infinite ternary expansion (½ = `0.+++…`), so the exact result of an
add, subtract, multiply or `fma`, which always has a finite expansion,
never ties. Only `fdiv` and `fsqrt` can, and they break ties toward
zero.

**Comparisons.** IEEE 754 defines four relations between floats (less,
equal, greater, unordered) and a trit holds three. The two questions
programs actually ask each fit a trit:

- `feq`, `flt` and `fle` are predicates returning `+` true, `-` false,
  and `0` when an operand is NaN. Unordered is Kleene's unknown, so the
  predicates combine correctly under `tmin`/`tmax`, and C maps exactly:
  `if (a < b)` is `flt` then a branch when the result is above 0, and
  `if (!(a < b))` branches when it is 0 or below, which IEEE requires to
  be taken for NaN. `flt` and `fle` raise the invalid flag on a NaN;
  `feq` does not.
- `fcmp` is the three-way compare for sorting and searching. It uses the
  IEEE 754 total order adapted to this format: NaNs with a negative
  significand sort below −∞, other NaNs above +∞, and NaNs among
  themselves by significand. It raises no flags and always returns −1,
  0 or +1, so `br3` takes its result directly.

**`fclass`.** Returns one of: −4 −∞, −3 negative normal, −2 negative
subnormal, 0 zero, +2 positive subnormal, +3 positive normal, +4 +∞,
+1 NaN. The sign of the class is the sign of the value.

**Flags.** `fcsr` (8.2) records invalid, divide-by-zero, overflow,
underflow and inexact, as `+` trits that stay set until cleared, plus
the dynamic rounding mode.

### 7.12 System (I, R)

| Op  | Mnemonic | Semantics |
|-----|----------|-----------|
| −30 | `sys`                     | trap with cause +4 (system call) |
| −31 | `tret`                    | return from trap (8.4) |
| −32 | `csrrw rd, csr, rs1`      | `rd = csr`; `csr = rs1` |
| −33 | `csrrtmax rd, csr, rs1`   | `rd = csr`; `csr = tmax(csr, rs1)`, tritwise |
| −34 | `csrrtmin rd, csr, rs1`   | `rd = csr`; `csr = tmin(csr, rs1)`, tritwise |
| −35 | `fence pred, succ`        | order the `pred` accesses before the `succ` accesses |
| −36 | `fence.i`                 | make earlier stores visible to instruction fetch |
| −37 | `fence.vm rs1, rs2`       | flush translations for address `rs1` and address space `rs2`; `r0` means all (9.7) |
| −38 | `wfi`                     | wait for an interrupt (may be a no-op) |
| −39 | `brk`                     | trap with cause +5 (breakpoint) |
| −40 | `cclean rs1`              | write back the cache line holding `rs1` |
| −41 | `cflush rs1`              | write back and invalidate the line |
| −42 | `cinval rs1`              | invalidate the line without writing back (ring 0 and above) |

CSR instructions use format I with the CSR number in `imm[8:0]`. When
the `rs1` field names `r0`, the CSR is read and not written, which gives
the read form `csrr rd, csr`; to write zero, use another register that
holds 0.

Interrupt-enable and similar CSR fields use `+` for on and `-` for off,
so `csrrtmax` with a mask of `+` on the chosen trits and `-` elsewhere
sets them, and `csrrtmin` with `-` on the chosen trits and `+` elsewhere
clears them, both atomically.

`fence` takes `pred` in `imm[3:0]` and `succ` in `imm[7:4]`, one trit
each for RAM reads, RAM writes, MMIO input and MMIO output, `+` to
include. A bare `fence` orders everything. `fence.i` is what a loader or
JIT runs before executing code it just wrote. The cache instructions are
for drivers that share buffers with devices.

### 7.13 Pseudo-instructions

| Pseudo | Expansion |
|--------|-----------|
| `nop`             | `addi r0, r0, 0` |
| `mv rd, rs`       | `addi rd, rs, 0` |
| `li rd, imm`      | `addi`, or `lui` + `addi` |
| `la rd, symbol`   | `auipc` + `addi` |
| `neg rd, rs` / `not rd, rs` | `sub rd, r0, rs` (the same operation) |
| `sgn rd, rs`      | `cmp rd, rs, r0` |
| `j label`         | `jal r0, label` |
| `call label`      | `auipc` + `jalr ra`, or `jal ra` when in reach |
| `ret`             | `jalr r0, 0(ra)` |
| `csrr rd, csr`    | `csrrw rd, csr, r0` |
| `tmin` etc.       | `tlut` with the table from 7.5 |

## 8. Privilege, traps and interrupts

### 8.1 Rings

Privilege is a single trit, the **ring**:

| Ring | Role |
|------|------|
| −1   | user |
| 0    | supervisor: the kernel |
| +1   | machine: firmware or hypervisor |

The current ring is internal state, not a CSR. Ring +1 always runs with
translation off. Rings 0 and −1 run translated when `ptbr` enables it.

### 8.2 CSRs

A CSR number is 9 trits: `number = 6561 × ring + 2187 × access + index`,
where `ring` (trit 8) is the lowest ring allowed to access it, `access`
(trit 7) is `+` for read/write and `-` for read-only, and `index` is the
low 7 trits. Access rights fall out of the number, with no lookup table.
Writing a read-only CSR, or touching one from too low a ring, is an
illegal instruction.

Rings 0 and +1 each have their own **trap bank**, the same indices at a
different ring trit. That is how one trap handler can run in the kernel
while the firmware's state stays untouched.

| Index | Name      | Contents |
|-------|-----------|----------|
| 0     | `tstatus` | trap state for this ring (below) |
| 1     | `tvec`    | trap handler address and mode |
| 2     | `tpc`     | `pc` at the trap |
| 3     | `tcause`  | trap cause |
| 4     | `tval`    | faulting address or instruction |
| 5     | `scratch` | free for the handler |
| 6     | `ie`      | interrupt enables: trit `i` enables cause −`i` |
| 7     | `ip`      | interrupts pending: trit `i` for cause −`i` |
| 8     | `timecmp` | timer compare value |

Ring +1 bank numbers are 8748 + index (8748 .. 8756); ring 0 bank
numbers are 2187 + index (2187 .. 2195). Assembly names them with an `m` prefix for
ring +1 and `s` for ring 0: `mtvec`, `stvec`, and so on.

Other CSRs:

| CSR       | Number | Ring | Access | Contents |
|-----------|--------|------|--------|----------|
| `edeleg`  | 8768   | +1   | rw | exception delegation, trit `i` for cause +`i` |
| `ideleg`  | 8769   | +1   | rw | interrupt delegation, trit `i` for cause −`i` |
| `hartid`  | 4374   | +1   | ro | this hart's number, balanced (5.6) |
| `ptbr`    | 2197   | 0    | rw | page-table base and mode (9.2) |
| `entropy` | −2187  | 0    | ro | fresh random trits from a hardware source |
| `cycle`   | −8748  | −1   | ro | this hart's cycles since it started |
| `time`    | −8747  | −1   | ro | wall-clock ticks, one clock shared by every hart, rate set by the platform |
| `instret` | −8746  | −1   | ro | instructions this hart has retired since it started |
| `fcsr`    | −4374  | −1   | rw | float flags (trits 4..0: invalid, divide-by-zero, overflow, underflow, inexact) and rounding mode (trits 6..5) |

`tstatus` fields:

| Trit | Name  | Meaning |
|------|-------|---------|
| 0    | `PR`  | ring before the last trap into this ring |
| 1    | `IE`  | interrupts for this ring: `+` enabled, `-` disabled |
| 2    | `PIE` | `IE` before the last trap |
| 3    | `SUA` | ring 0 bank only: `+` lets the kernel access user pages |
| 4    | `SS`  | single step: `+` makes `tret` run one instruction, then trap with cause +9 |

`tvec` trit 0 selects the mode, and the rest is the handler address
with that trit taken as 0:

- `0`, direct: every trap goes to the address.
- `+`, vectored: a trap goes to `address + 3 × cause`. Exceptions land
  above the address and interrupts below it, one instruction slot each,
  so one table centred on the address serves both kinds.

### 8.3 Delegation

Every trap starts targeted at ring +1. A delegation trit of `+` for its
cause sends it to ring 0 instead. `0` keeps it at ring +1, and `-` is
reserved, perhaps for user-level handlers later. Reset leaves both
delegation CSRs zero, so firmware sees everything until it hands traps
to the kernel. A trap never goes to a ring below the current one: a
delegated trap raised in ring +1 stays in ring +1.

### 8.4 Trap entry and return

On a trap into ring `r`, using ring `r`'s bank:

1. `tpc` = the trapping instruction (for an interrupt, the next one to
   run); `tcause` and `tval` are set.
2. `PR` = the current ring; `PIE` = `IE`; `IE` = `-`.
3. The current ring becomes `r`, and `pc` comes from `tvec`.

A `sys` handler resumes after the call by adding 3 to `tpc`.

`tret` returns using the bank of the current ring: `pc = tpc`,
`IE = PIE`, current ring = `PR`. It is legal in rings 0 and +1.

An interrupt with cause −`i` is taken into its target ring `r` when
`ip[i]` and `ie[i]` are both `+` in `r`'s bank, and either the current
ring is below `r`, or it equals `r` with `IE = +`. Ring `r`'s timer
interrupt is pending whenever `time ≥ timecmp`; software clears it by
moving `timecmp` forward. Each hart has its own `timecmp` registers and
so its own timer interrupt, all against the shared `time`.

### 8.5 Causes

The sign of the cause separates interrupts from exceptions, the job
RISC-V gives the top bit of `mcause`.

| Cause | Kind |
|-------|------|
| +1 | illegal instruction |
| +2 | misaligned access |
| +3 | access fault (nothing at that physical address, or a forbidden region) |
| +4 | system call, ring in `tstatus.PR` |
| +5 | breakpoint |
| +6 | instruction page fault |
| +7 | load page fault |
| +8 | store page fault |
| −1 | timer interrupt |
| −2 | external interrupt: devices, and the hart controller announcing a hart added or a removal requested |
| −3 | software interrupt: this hart's mailbox holds a message (5.6), or software set `ip` trit 3 |
| +9 | single step |

### 8.6 Debug

The kernel can debug programs, and firmware can debug the kernel, with
**triggers**: nine pairs of ring 0 CSRs, `tdaddr0`..`tdaddr8`
(2207..2215) and `tdctl0`..`tdctl8` (2217..2225).

| `tdctl` trit | Meaning |
|--------------|---------|
| 0 | `+`: match instruction fetch |
| 1 | `+`: match loads |
| 2 | `+`: match stores |
| 3 | `+`: match in ring −1 |
| 4 | `+`: match in ring 0 |
| 5 | match size: `0` the tryte, `+` the aligned word, `-` the page |
| 6 | action: `0` off, `+` breakpoint trap, `-` reserved for an external debugger |

A matching fetch traps before the instruction runs, and a matching load
or store traps before the access, with cause +5 and the address in
`tval`. Together with `SS` (8.2) this is what a debugger needs:
breakpoints without rewriting code, watchpoints, and stepping.

The emulator has its own debugger as well, outside the machine, which
needs none of this.

## 9. Virtual memory

### 9.1 Address split

A virtual address splits by cutting trits, with no arithmetic:

```
trit:  26 25 | 24 ....... 17 | 16 ........ 9 | 8 ........ 0
       root  | mid index (8) | leaf index (8)| page offset (9)
```

A page is 3⁹ = 19,683 trytes, centred on a multiple of 3⁹. Mid and leaf
tables hold 3⁸ = 6,561 one-word entries and so fill exactly one page
each. The root table has only 9 entries, 27 trytes: it fits in a single
cache line, which sits at the centre of the page `ptbr` names. The rest
of that page is free for the kernel, for example to hold the process's
own bookkeeping, so an address space costs very little until it maps
something.

Indices are signed, and a table is centred like everything else: entry
`i` is the word at `table + 3 × i`. A walk takes three steps, root, mid,
leaf, and the TLB hides them.

The sign of the root index gives a natural split, with the kernel in
negative addresses and user space in positive ones. Virtual page 0 is
left unmapped, so a null pointer (or anything within 9,841 trytes of one)
faults.

### 9.2 `ptbr`

| Trits  | Field | Meaning |
|--------|-------|---------|
| 0      | mode  | `0`: no translation; `+`: three-level paging; `-`: reserved |
| 8..1   | ASID  | address-space identifier (3⁸ of them), tags TLB entries |
| 26..9  | root  | physical page number of the page whose centre holds the root table |

A write that would set the reserved mode leaves `ptbr` unchanged.

### 9.3 Page-table entries

| Trits  | Field | Meaning |
|--------|-------|---------|
| 0      | `V`   | `0` invalid, `-` points to the next-level table, `+` maps a page |
| 1      | `R`   | `+` readable |
| 2      | `W`   | `+` writable |
| 3      | `X`   | `+` executable |
| 4      | `U`   | `+` user-accessible; `0` kernel only |
| 5      | `G`   | `+` global: present in every address space |
| 6      | `A`   | `+` accessed |
| 7      | `D`   | `+` dirty |
| 8      | `S`   | free for the operating system |
| 26..9  | `PPN` | physical page number |

One trit says both whether an entry is valid and whether it is a leaf
or a pointer, where RISC-V needs a valid bit plus the convention that
`R = W = X = 0` means "pointer". A zeroed table is all invalid entries.

### 9.4 The walk

A walk reads the root entry, then the mid entry, then the leaf entry,
stopping at the first `+` entry, which is the leaf. Page-table reads are
physical: they must land in RAM or the boot page, and one that lands in
device memory, or nowhere, is an access fault (cause +3).

A `+` entry in a mid table maps a superpage of 3¹⁷ trytes (3⁸ pages),
and one in the root table maps 3²⁵ trytes. A superpage's `PPN` must be
aligned to its size: its low 8 trits are 0 for a mid-level leaf, its low
16 for a root-level one. Virtual page `vpn` within the superpage maps to
physical page `PPN + k`, where `k` is the part of `vpn` the leaf does not
index.

These entries are reserved and make the walk fault:

- a `-` (pointer) entry in a leaf table, which has no level below it;
- a pointer with any of trits 1..7 nonzero: permissions belong to leaves;
- a leaf with `-` in any of trits 1..7;
- a superpage whose `PPN` is not aligned.

The `S` trit is never checked.

### 9.5 Permissions

An access needs, from its leaf:

| Access | Needs |
|--------|-------|
| instruction fetch | `X` |
| load, `lr` | `R` |
| store, `sc` | `W` |
| AMO, `cas` | `R` and `W` |

and in every case `A = +`, plus `D = +` for anything that writes.

The `U` trit divides the rings. Ring −1 may only touch pages with `U = +`.
Ring 0 never executes a `U = +` page, and loads or stores to one only
when `SUA` is set in its `tstatus`, so the kernel cannot follow a user
pointer by accident. Ring +1 is never translated.

The hardware does not set `A` or `D`. An access to a page whose `A` is
not `+`, or a store to one whose `D` is not `+`, takes a page fault, and
the kernel sets the trit. That keeps the walker simple and gives the
kernel exact working-set information.

### 9.6 Page faults

A failed walk or permission check is a page fault: cause +6 for an
instruction fetch, +7 for a load or `lr`, and +8 for a store, `sc`, AMO
or `cas` (an AMO faults as a store even when it lacks only `R`). `tval`
holds the faulting virtual address. An access fault (cause +3) also
reports the virtual address.

Alignment is checked before translation. Since a word never crosses a
page (5.2), one translation always covers a whole access.

### 9.7 TLB and `fence.vm`

An implementation may cache the leaf entry of any translation that
succeeded, tagged with the current ASID, or with no ASID when `G = +`,
and may keep it until `fence.vm` removes it. It never caches a walk that
faulted. So after changing an entry the kernel runs `fence.vm`, and
switching `ptbr` to a different ASID needs none.

`fence.vm rs1, rs2` removes cached translations:

| `rs1` | `rs2` | Removes |
|-------|-------|---------|
| `r0` | `r0` | everything |
| address | `r0` | that page, in every address space, global entries included |
| `r0` | ASID | that address space's entries; global entries stay |
| address | ASID | that page in that address space; global entries stay |

`r0` here means the register field names `r0`, not a register holding 0:
ASID 0 is named with a register that holds 0.

`fence.vm` acts only on the hart that runs it; other harts are flushed
by asking them to run it (5.6).

## 10. Example: binary search with a three-way branch

```asm
# a0 = address of a sorted array of words, a1 = length, a2 = key.
# Returns the index of key in a0, or -1.
bsearch:
        li    t0, 0              # lo
        addi  t1, a1, -1         # hi
        li    t3, 2
loop:
        bgt   t0, t1, notfound
        add   t2, t0, t1
        div   t2, t2, t3         # mid: halving is a real division here
        shi   t4, t2, 1          # word index to tryte offset: x3 is one shift
        add   t4, t4, a0
        lw    t5, 0(t4)
        cmp   t6, a2, t5         # sign(key - a[mid])
        br3   t6, left, right    # -: go left, +: go right, 0: found
        mv    a0, t2
        ret
left:   addi  t1, t2, -1
        j     loop
right:  addi  t0, t2, 1
        j     loop
notfound:
        li    a0, -1
        ret
```

This shows both sides of the trade. The three-way outcome of the key
comparison costs one `cmp` and one `br3`, but the midpoint costs a
`div`, which a binary machine does with a shift. A ternary search that
splits the range in thirds would make that division a shift.

## 11. Living with binary

Files, networks and peripherals are binary. The ISA's part:

- The binary operations of 7.6 handle bit-specified algorithms.
- A device moves data as one byte per tryte (values 0..255), through
  MMIO in the negative page range. This wastes about 44% of each tryte
  and keeps drivers simple.
- For storage and transport, 5 trits pack into a byte (3⁵ = 243 ≤ 256),
  which wastes about 5%. Packing is done in software for now.

## 12. Sketches (not yet normative)

### 12.1 Vectors

A scalable vector extension in the style of RISC-V V or Arm SVE, with
packed trytes or words per lane. The third trit of a mask lane means
"inactive, leave unchanged", distinct from false.

### 12.2 Text

A tryte has 19,683 values, too few for all of Unicode (1,114,112 code
points) in one unit. A variable-length encoding can use the sign of the
first tryte: a non-negative tryte is a code point from 0 to 9,841
directly, which covers ASCII, Latin, Greek, Cyrillic, Hebrew, Arabic and
the main Indic scripts, and a negative tryte starts a two-tryte
sequence, whose 18 trits cover every code point with room to spare.

### 12.3 Platform

`platform.md` fixes the reference platform's MMIO devices: a console,
power control and the hart controller (5.6). Still to come: an
interrupt controller for devices and a block device, at negative page
numbers like the rest.

## 13. Open questions

1. **`br3` reach.** Eight-trit offsets (±3,280 words) suit local
   dispatch; one longer offset may be worth more.
2. **Compressed coverage.** The slot assignments in 6.1 are a first
   guess, to be revisited against measured instruction frequencies.
3. **Hypervisor support.** Whether ring +1 should grow two-stage
   translation so detrytus can host detrytus guests.
4. **Waiting on a word.** Proposed as `wrs` (7.9), after RISC-V's
   Zawrs: a hart sleeps until the word it reserved with `lr` is written,
   an interrupt is pending, or a deadline passes. Open until accepted,
   and with it: whether the deadline operand earns its place (Zawrs has
   two instructions instead, with and without a short timeout), and
   whether ring 0 should be able to make `wrs` in user mode trap, as
   RISC-V's `TW` bit does for `wfi`.
5. **Removing hart 0.** Whether the boot hart may ever be removed, once
   another hart has taken over its duties.
