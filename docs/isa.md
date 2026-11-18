# detrytus ISA

Version 0.1, **draft**. Everything here may change until the emulator
has run real programs against it. Sections marked *sketch* are not yet
normative: they record the direction, not the rule.

detrytus is a balanced-ternary instruction set in the spirit of the
Soviet Setun (1958) and Setun-70 (1970), reshaped around what a modern
RISC core looks like: a load/store architecture, a fixed instruction
width, a hardwired zero register, privilege rings, traps, CSRs and
atomics. Where ternary lets a feature be simpler than its binary
equivalent, this spec takes the simpler form and says so.

## 1. Notation

A **trit** takes one of three values: −1, 0, +1, written `-`, `0`, `+`.

A sequence of trits is written most significant first. In assembly a
balanced-ternary literal carries the prefix `0t`, the way `0b` marks
binary: `0t+0-` is 9 − 1 = 8. Decimal literals are also accepted and
are the default in this document.

Trit positions are numbered from 0 at the least significant end. `x[i]`
is trit `i` of `x`; `x[j:i]` is the field from trit `j` down to trit `i`
inclusive.

`wrap(x)` reduces an integer into the 27-trit word range by arithmetic
modulo 3²⁷, choosing the representative in
[−3,812,798,742,493, +3,812,798,742,493].

## 2. Why balanced ternary changes the design

These properties are what the rest of the spec leans on.

- **No sign bit, no unsigned types.** The sign of a number is the sign
  of its most significant nonzero trit. Every value is signed, and there
  is exactly one kind of integer. There are no unsigned compares, no
  unsigned multiplies, no zero-extend versus sign-extend loads.
- **Negation is tritwise inversion.** Flipping every trit negates the
  value, so `NOT` and `NEG` are the same operation and the range is
  symmetric: there is no most-negative number that cannot be negated.
- **Truncation rounds to nearest.** Dropping low trits leaves the
  nearest integer to the exact quotient. A right shift is therefore a
  correctly rounded division by a power of three, and there is no
  arithmetic versus logical shift distinction.
- **Comparison never overflows.** Ordering is lexicographic on trits,
  most significant first, with `-` < `0` < `+`. A compare does not need
  a subtraction and so cannot overflow.
- **A compare has three outcomes.** It yields a trit, which is itself a
  first-class value: it can be stored, branched on three ways, or used
  to select.
- **Constants split without a carry fix-up.** Any integer is uniquely
  `hi × 3ⁿ + lo` with `lo` in the n-trit balanced range, so building a
  constant from an upper and a lower immediate needs none of RISC-V's
  `%hi` rounding adjustment.

The costs are real too. Halving is not a shift: dividing by two is a
true division, so algorithms that lean on powers of two (binary search
midpoints, buddy allocators, hash masks) pay for it. Everything outside
the core speaks binary, which section 12 discusses.

## 3. Data types

| Unit  | Trits | Range                         | Approx. binary equivalent |
|-------|-------|-------------------------------|---------------------------|
| trit  | 1     | −1 .. +1                      | 1.58 bits                 |
| tryte | 9     | −9,841 .. +9,841              | 14.3 bits                 |
| word  | 27    | ±3,812,798,742,493            | 42.8 bits                 |

The word is three trytes. The 9/27 sizes keep every field boundary on a
power of three, which is why 27 was chosen over a 41-trit word that would
match a 64-bit range.

## 4. Programmer-visible state

### 4.1 Registers

There are 27 general-purpose registers, each one word, named by a
balanced 3-trit index from −13 to +13: `r-13` .. `r-1`, `r0`, `r1` ..
`r13`. `r0` always reads as zero and ignores writes.

The program counter `pc` is a separate word register. It always holds
a multiple of 3 (see 5.2).

### 4.2 Calling convention (provisional)

| Register      | ABI name    | Role                      | Saved by |
|---------------|-------------|---------------------------|----------|
| `r0`          | `zero`      | constant zero             |          |
| `r1`..`r6`    | `a0`..`a5`  | arguments, `a0` returns   | caller   |
| `r7`..`r13`   | `t0`..`t6`  | temporaries               | caller   |
| `r-1`..`r-10` | `s0`..`s9`  | saved                     | callee   |
| `r-11`        | `fp`        | frame pointer             | callee   |
| `r-12`        | `ra`        | return address            | caller   |
| `r-13`        | `sp`        | stack pointer             | callee   |

The split by sign is deliberate: positive registers are scratch,
negative ones survive a call. The stack grows toward negative addresses.

## 5. Memory

### 5.1 Addressing

Memory is tryte-addressable. An address is a word, so addresses are
signed and the address space runs from −3²⁷/2 to +3²⁷/2, centred on 0.
That is about 7.6 × 10¹² trytes.

### 5.2 Alignment and byte order

A word occupies three consecutive trytes and must be **aligned**: its
address must be a multiple of 3. In balanced ternary that is exactly the
condition "trit 0 of the address is 0", so the alignment check is a
single trit test. A misaligned word access traps (cause 2).

Words are stored **little-endian**: at address `a`, tryte `a` holds trits
8..0, `a+1` holds 17..9, `a+2` holds 26..18.

Instructions are words, so `pc` is always aligned and branch offsets
count instructions, not trytes.

### 5.3 Reset

On reset, all registers are 0, `pc` is 0, the current ring is +1 and
interrupts are disabled. The all-zero word is an illegal instruction
(opcode 0, section 6), so execution that runs into zeroed
memory traps instead of sliding.

## 6. Instruction encoding

Every instruction is one 27-trit word. The opcode sits in the low five
trits, giving 243 major opcodes. Opcode 0 is permanently illegal. Each
instruction has its own major opcode; there is no secondary function
field except where noted, which keeps decode a single table lookup.

Register fields keep the same position in every format that has them,
so the register file can be read before the format is known.

```
trit:     26 ............ 17 | 16  14 | 13  11 | 10   8 | 7   5 | 4   0
R         fn[10]             | rs3    | rs2    | rs1    | rd    | op
I         imm[16] ...................          | rs1    | rd    | op
S         imm[16] ...................          | rs1    | rs2   | op
B         off[13] ............      | rs2    | rs1    | h c c | op
T         offP[8] ....  | offN[8] ....   | rs1    | 0 0 h | op
U         imm[19] ....................................  | rd    | op
J         off[19] ....................................  | rd    | op
```

| Format | Fields                                                        |
|--------|---------------------------------------------------------------|
| R      | `rd`, `rs1`, `rs2`, `rs3`, `fn` (10 trits)                    |
| I      | `rd`, `rs1`, `imm` (16 trits, ±21,523,360)                    |
| S      | `rs2` (in the `rd` slot), `rs1`, `imm` (16 trits)             |
| B      | `cond` (trits 6..5), hint `h` (trit 7), `rs1`, `rs2`, `off` (13 trits, ±797,161 instructions) |
| T      | hint `h` (trit 5), trits 7..6 zero, `rs1`, `offN`, `offP` (8 trits each, ±3,280 instructions) |
| U      | `rd`, `imm` (19 trits)                                        |
| J      | `rd`, `off` (19 trits, ±581,130,733 instructions)             |

Unused register fields must be `r0`; unused `fn` trits must be 0. A
nonzero value in a must-be-zero field is an illegal instruction, so the
space stays available for extensions.

Branch and jump targets are `pc + 3 × off`, relative to the address of
the branch itself.

## 7. Base instruction set

`rd`, `rs1` etc. below mean the register values. All arithmetic results
pass through `wrap`.

### 7.1 Arithmetic (R)

| Op | Mnemonic | Semantics |
|----|----------|-----------|
| 1  | `add rd, rs1, rs2`  | `rd = wrap(rs1 + rs2)` |
| 2  | `sub rd, rs1, rs2`  | `rd = wrap(rs1 − rs2)` |
| 3  | `mul rd, rs1, rs2`  | low word of the 54-trit product |
| 4  | `mulh rd, rs1, rs2` | high word of the 54-trit product |
| 5  | `div rd, rs1, rs2`  | quotient, truncated toward zero |
| 6  | `rem rd, rs1, rs2`  | remainder, sign of `rs1`; `rs1 = div × rs2 + rem` |

The 54-trit product splits uniquely as `hi × 3²⁷ + lo` with both halves
in word range, so one `mulh` covers what RISC-V needs `mulh`, `mulhu`
and `mulhsu` for.

Division cannot overflow, because the range is symmetric. Division by
zero does not trap: `div` gives 0 and `rem` gives `rs1`, which keeps the
identity `rs1 = div × rs2 + rem` true.

### 7.2 Compare and select (R)

| Op | Mnemonic | Semantics |
|----|----------|-----------|
| 7  | `cmp rd, rs1, rs2`      | `rd = sign(rs1 − rs2)`, computed without overflow |
| 11 | `sel rd, rs1, rs2, rs3` | `rd = rs2` if `rs1 < 0`; `rd` unchanged if `rs1 = 0`; `rd = rs3` if `rs1 > 0` |

`cmp` always yields −1, 0 or +1. `sel` takes a three-way condition and
reads `rd` as its middle input. That costs a fourth register read port;
see open question 2.

### 7.3 Shifts (R)

| Op | Mnemonic | Semantics |
|----|----------|-----------|
| 8  | `sh rd, rs1, rs2`  | shift left by `rs2` trits; a negative `rs2` shifts right |
| 9  | `rot rd, rs1, rs2` | rotate left by `rs2` trits (taken modulo 27); negative rotates right |

A left shift by `n` is `wrap(rs1 × 3ⁿ)`. A right shift by `n` drops the
low `n` trits, which is `rs1 / 3ⁿ` rounded to nearest. Ties cannot occur
because 3ⁿ is odd. Shift amounts of 27 or more in either direction give
0. One instruction with a signed amount replaces the usual three
(left, logical right, arithmetic right).

### 7.4 Tritwise logic (R)

| Op | Mnemonic | Semantics |
|----|----------|-----------|
| 10 | `tlut rd, rs1, rs2, table` | for each trit `i`: `rd[i] = table[3·(rs1[i]+1) + (rs2[i]+1)]` |

`table` is a 9-trit truth table held in `fn[8:0]` (`fn[9]` must be 0).
Entry `k` of the table is the output for the input pair
`(a, b) = (k div 3 − 1, k mod 3 − 1)`, so entry 0 is `(-,-)` and entry 8
is `(+,+)`. One instruction expresses all 19,683 two-input trit
functions, in the same way AVX-512's `vpternlog` covers every
three-input boolean function.

The assembler provides names for the common tables:

| Pseudo | Function | Table (decimal) | Table (`0t`, entry 8 first) |
|--------|----------|-----------------|------------------------------|
| `tmin`  | Kleene AND, `min(a, b)`          | 5792  | `0t+0-00----` |
| `tmax`  | Kleene OR, `max(a, b)`           | 9728  | `0t++++00+0-` |
| `tmul`  | `a × b` (the ternary XNOR)       | 5824  | `0t+0-000-0+` |
| `tsum`  | `a + b` modulo 3, no carry       | −4160 | `0t-+0+0-0-+` |
| `tcons` | consensus: `a` if `a = b`, else 0 | 6560 | `0t+0000000-` |
| `tany`  | accept-anything: `sign(a + b)`   | 8960  | `0t++0+0-0--` |

Unary functions use `rs2 = r0`, so only entries 1, 4 and 7 matter. The
pseudo `tmap rd, rs, f` takes any of the 27 unary trit functions; for
example `tabs` (9490) maps every trit to its magnitude. Negation is not
listed because it is arithmetic `neg`.

Kleene logic gives a natural "unknown" value. A trit vector can carry
true/false/unknown per lane, the shape of SQL `NULL` semantics or an
initialised/uninitialised shadow map, and `tmin`/`tmax` propagate the
unknowns correctly.

### 7.5 Immediates (I, U)

| Op | Mnemonic | Format | Semantics |
|----|----------|--------|-----------|
| 13 | `addi rd, rs1, imm` | I | `rd = wrap(rs1 + imm)` |
| 14 | `cmpi rd, rs1, imm` | I | `rd = sign(rs1 − imm)` |
| 15 | `shi rd, rs1, imm`  | I | `sh` by an immediate amount |
| 16 | `roti rd, rs1, imm` | I | `rot` by an immediate amount |
| 17 | `lui rd, imm`       | U | `rd = imm × 3⁸` |

`lui` followed by `addi` with an 8-trit immediate builds any word
constant, with no carry correction (section 2).

### 7.6 Loads and stores (I, S)

| Op | Mnemonic | Format | Semantics |
|----|----------|--------|-----------|
| −1 | `lt rd, imm(rs1)`  | I | load the tryte at `rs1 + imm`; the upper trits are filled with 0 |
| −2 | `lw rd, imm(rs1)`  | I | load the aligned word at `rs1 + imm` |
| −3 | `st rs2, imm(rs1)` | S | store trits 8..0 of `rs2` to the tryte at `rs1 + imm` |
| −4 | `sw rs2, imm(rs1)` | S | store `rs2` to the aligned word at `rs1 + imm` |

Filling with zero trits preserves the value, so there is one tryte load,
not a signed and an unsigned pair.

### 7.7 Control transfer (B, T, J, I)

| Op  | Mnemonic | Format | Semantics |
|-----|----------|--------|-----------|
| −10 | `jal rd, label`       | J | `rd = pc + 3`; `pc = pc + 3 × off` |
| −11 | `jalr rd, imm(rs1)`   | I | `rd = pc + 3`; `pc = rs1 + imm` (must be aligned) |
| −12 | `b<cond> rs1, rs2, label` | B | if `cmp(rs1, rs2)` satisfies `cond`, `pc = pc + 3 × off` |
| −13 | `br3 rs1, Lneg, Lpos` | T | `rs1 < 0`: `pc += 3 × offN`; `rs1 > 0`: `pc += 3 × offP`; `rs1 = 0`: fall through |

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

The ordered conditions are arranged so that negating `cond` mirrors the
comparison, which is the same as swapping the operands.

**Static prediction hints.** Formats B and T carry one spare trit used
as a hint, which costs nothing in a 27-trit word. In `b<cond>` the hint
is −1 for "unlikely taken", 0 for no hint, +1 for "likely taken". In
`br3` the hint is the predicted sign of `rs1`. Hints never change
behaviour. Assembler syntax: an optional `.likely` / `.unlikely` suffix
on `b<cond>`, and `.n` / `.z` / `.p` on `br3`.

`br3` is the instruction Setun's design points at: one compare, one
branch, three destinations. Its offsets are short, so a far target goes
through a `jal` trampoline.

### 7.8 Atomics (R)

All take the address in `rs1` with no offset, and it must be word
aligned. Each is a single indivisible read-modify-write.

| Op  | Mnemonic | Semantics |
|-----|----------|-----------|
| −20 | `amoswap rd, rs2, (rs1)` | `rd = mem[rs1]`; `mem[rs1] = rs2` |
| −21 | `amoadd rd, rs2, (rs1)`  | `rd = mem[rs1]`; `mem[rs1] = wrap(rd + rs2)` |
| −22 | `amomin rd, rs2, (rs1)`  | `rd = mem[rs1]`; `mem[rs1] = min(rd, rs2)` |
| −23 | `amomax rd, rs2, (rs1)`  | `rd = mem[rs1]`; `mem[rs1] = max(rd, rs2)` |
| −24 | `cas rd, rs2, rs3, (rs1)` | `rd = mem[rs1]`; if `rd = rs2` then `mem[rs1] = rs3` |

Because every integer is signed, `amomin` and `amomax` need no unsigned
variants.

### 7.9 System (I, R)

| Op  | Mnemonic | Semantics |
|-----|----------|-----------|
| −30 | `sys`               | trap with cause 4 (system call) |
| −31 | `tret`              | return from trap (section 8) |
| −32 | `csrrw rd, csr, rs1` | `rd = csr`; `csr = rs1` (I format, CSR number in `imm[8:0]`) |
| −33 | `fence`             | order all earlier memory accesses before all later ones |
| −34 | `wfi`               | wait for an interrupt (may be a no-op) |
| −35 | `brk`               | trap with cause 5 (breakpoint) |

The trits of `imm` above the CSR number must be 0. When the `rs1`
field names `r0`, the CSR is read and not written, which gives the read
form `csrr rd, csr`; to write zero, use any other register holding 0.

### 7.10 Pseudo-instructions

| Pseudo | Expansion |
|--------|-----------|
| `nop`            | `addi r0, r0, 0` |
| `mv rd, rs`      | `addi rd, rs, 0` |
| `li rd, imm`     | `addi rd, r0, imm`, or `lui` + `addi` for large values |
| `neg rd, rs` / `not rd, rs` | `sub rd, r0, rs` (the same operation) |
| `sgn rd, rs`     | `cmp rd, rs, r0` |
| `j label`        | `jal r0, label` |
| `call label`     | `jal ra, label` |
| `ret`            | `jalr r0, 0(ra)` |
| `tmin` etc.      | `tlut` with the table from 7.4 |

Numeric `min` needs a scratch register, so it is an idiom rather than a
pseudo-instruction: `cmp t, a, b` ; `mv rd, a` ; `sel rd, t, a, b`. When
`a < b` or `a = b` the result stays `a`, and when `a > b` it becomes `b`.

## 8. Privilege, traps and interrupts

### 8.1 Rings

Privilege is a single trit, the **ring**:

| Ring | Role |
|------|------|
| −1   | user |
| 0    | supervisor (kernel) |
| +1   | machine / hypervisor |

Three levels is roughly where x86 and ARM ended up in practice, and here
it is the natural width of the field. In version 0.1 every trap is taken
into ring +1; delegating traps to ring 0 is open question 3.

### 8.2 CSRs

A CSR number is 9 trits, and its top two trits encode the access rule,
so no table is needed to check a CSR access:

- trit 8: the minimum ring allowed to access it.
- trit 7: `+` read/write, `-` read-only, `0` reserved.

Writing a read-only CSR, or accessing one from too low a ring, is an
illegal instruction.

| CSR       | Number | Ring | Access | Contents |
|-----------|--------|------|--------|----------|
| `status`  | 8748   | +1   | rw | ring and interrupt state (below) |
| `tvec`    | 8749   | +1   | rw | trap handler address |
| `tpc`     | 8750   | +1   | rw | `pc` at the trap |
| `tcause`  | 8751   | +1   | rw | trap cause |
| `tval`    | 8752   | +1   | rw | faulting address or instruction |
| `scratch` | 8753   | +1   | rw | free for the trap handler |
| `cycle`   | −8748  | −1   | ro | cycles since reset |
| `instret` | −8747  | −1   | ro | instructions retired since reset |

`status` fields:

| Trit | Name | Meaning |
|------|------|---------|
| 0    | `CR`  | current ring |
| 1    | `PR`  | ring before the last trap |
| 2    | `IE`  | interrupts: `+` enabled, `-` disabled, `0` reserved |
| 3    | `PIE` | `IE` before the last trap |

### 8.3 Trap entry and return

On a trap: `tpc` = address of the trapping instruction (for interrupts,
of the next instruction to run); `tcause` and `tval` are set; `PR = CR`;
`CR = +1`; `PIE = IE`; `IE = -`; `pc = tvec`. A `sys` handler resumes
after the call by adding 3 to `tpc`.

`tret` sets `pc = tpc`, `CR = PR`, `IE = PIE`. It is legal only in ring
+1.

### 8.4 Causes

The sign of `tcause` distinguishes the kind of trap, the job RISC-V
gives the top bit of `mcause`:

| Cause | Kind |
|-------|------|
| +1 | illegal instruction |
| +2 | misaligned access |
| +3 | access fault |
| +4 | system call (`sys`), ring in `status.PR` |
| +5 | breakpoint (`brk`) |
| −1 | timer interrupt |
| −2 | external interrupt |
| −3 | software interrupt |

How the timer and devices are reached (memory-mapped registers, their
addresses) belongs to the platform description, not to the ISA, and is
left to the emulator for now.

## 9. Example: binary search with a three-way branch

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

The example shows both sides of the trade: the three-way outcome of the
key comparison costs one `cmp` and one `br3`, but the midpoint costs a
`div`, which a binary machine does with a shift. A ternary search that
splits the range in thirds would turn that division into a shift.

## 10. Sketches (not yet normative)

### 10.1 Virtual memory

A page is 3⁹ = 19,683 trytes (6,561 words). A page-table entry is one
word, so a page holds 3⁸ entries. A 27-trit virtual address splits as a
10-trit root index, an 8-trit leaf index and a 9-trit page offset. The
root table is three contiguous pages (3¹⁰ entries), the same trick
RISC-V's hypervisor extension uses for its 16 KiB root. A natural
convention puts the kernel in negative addresses and user space in
positive ones, decided by the sign of the address.

### 10.2 Floating point

A balanced-ternary float needs no sign bit: the sign is the sign of the
significand. A 27-trit format might use a 6-trit exponent (±364) and a
21-trit significand normalised so its leading trit is nonzero. Rounding
to nearest is truncation. The open design work is the encoding of
infinities and NaN.

### 10.3 Vectors

Packed trytes in wide registers, with the third trit state of a mask
lane meaning "inactive, leave unchanged", distinct from "false".

### 10.4 Text

A tryte has 19,683 values, not enough for all of Unicode (1,114,112 code
points) in one unit. A variable-length encoding can use the sign of the
first tryte: a non-negative tryte is a code point from 0 to 9,841
directly (covering ASCII, Latin, Greek, Cyrillic, Hebrew, Arabic and the
main Indic scripts), and a negative tryte starts a two-tryte sequence,
whose 18 trits cover every code point with room to spare.

## 11. Open questions

1. **Division rounding.** `div` truncates toward zero for C
   compatibility. Round-to-nearest is what ternary does natively and
   would give a remainder in the balanced range. Add a `divn`?
2. **`sel` reads `rd`.** Four register reads per instruction is costly
   in hardware. The alternative is a zero middle case, which breaks the
   `min`/`max` idiom.
3. **Trap delegation.** How traps reach ring 0 without a duplicate set
   of CSRs per ring.
4. **`br3` reach.** Eight-trit offsets (±3,280 instructions) suit local
   dispatch. Is that enough, or should one offset be longer than the
   other?
5. **Compressed instructions.** A 9-trit or 18-trit short form, the way
   RISC-V has its C extension.
6. **Binary interoperability.** See section 12.

## 12. Living with binary

Files, networks and every peripheral are binary. Some options to settle
before the emulator does real I/O:

- Pack 5 trits into a byte (3⁵ = 243 ≤ 256) for storage and transport.
- Give devices a binary data path and convert at the boundary, with
  conversion instructions or a conversion unit.
- Define how a binary byte stream maps to trytes for things like file
  reads (one byte per tryte is simple and wastes about 44%).
