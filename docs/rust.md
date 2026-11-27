# Rust on detrytus

Version 0.1, **draft**. How a `no_std` Rust program runs on detrytus:
rustc compiles it as for a 32-bit binary machine and writes LLVM IR,
and `detrytus ll` translates that IR into detrytus assembly through the
back end Trice uses ([`trice.md`](trice.md)), following
[`abi.md`](abi.md). The binary integers Rust assumes are carried on the
ternary machine as described below. The reasons for this approach, and
the alternatives that were rejected, are in `CONTEXT.md` ("Rust through
LLVM IR").

## Building a program

rustc compiles for the built-in target `riscv32imac-unknown-none-elf`,
which has 32-bit pointers, a `usize` of 32 bits and little-endian data.
No RISC-V code is ever run: only the IR is used. `core` is built from
source alongside the program (`-Zbuild-std`, which `RUSTC_BOOTSTRAP=1`
allows on a stable toolchain), so that its IR is there too; with
`-Zbuild-std=core,alloc` and `alloc-*.ll` given too, a program may
use `alloc` (`Vec`, `String`, `Box`, `format!`) over a
`#[global_allocator]` it or its runtime defines. rustc then defines
`__rust_alloc` and the rest in the program's module; the marker
function its allocator shim would add at link time
(`__rust_no_alloc_shim_is_unstable_v2`) is taken as doing nothing.
Panics
abort at once (`-Cpanic=immediate-abort`): a panic becomes a call to
`abort` and none of core's formatting comes in with it.

```sh
rustup target add riscv32imac-unknown-none-elf
rustup component add rust-src
# a cargo project whose crate type is staticlib, release profile with
# opt-level = "s" (smaller than "z" here, since "z" keeps iterator
# methods out of line), panic = "abort", codegen-units = 1
RUSTC_BOOTSTRAP=1 \
RUSTFLAGS="--emit=llvm-ir,link -Zunstable-options -Cpanic=immediate-abort" \
cargo build --release -Zbuild-std=core --target riscv32imac-unknown-none-elf
# cargo leaves the IR in deps/ before 1.100, in build/CRATE/HASH/out/ since
ir=$(find target/riscv32imac-unknown-none-elf/release -name 'prog-*.ll' -o -name 'core-*.ll')
detrytus ll --include start.s $ir -o prog.s
```

`detrytus ll FILE.ll... [--include FILE]... [-o OUT]` reads the modules
together, keeps what `main` reaches and writes one assembly file (or an
image, for `-o OUT.dti`) that `.include`s each `--include` first. A
private name that two modules share (`@alloc_...`, `@vtable.0`) is kept
apart by giving the second module's a suffix (`.m1`). Names no module
defines are left to the included assembly. `--prune` writes the
reachable part of the modules as one LLVM IR module instead, which is
how the tests keep only what they need of core; `--root NAME` (any
number of times) prunes from those functions instead of `main`.
Three options are for a kernel (Words and the machine, below):
`--root NAME` (any number of times) keeps those functions and what they
reach instead of `main`, which a kernel does not have;
`--signed-pointers` makes pointers signed (Pointers); and
`--sections CODE,DATA` puts the code in section CODE and the static
data in DATA, sections the including assembly has placed, ending back
in CODE.

## What the program and its runtime agree on

The program defines `main` as

```rust
#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, argv: *const *const u8) -> i32
```

and the included assembly provides:

- start code that calls it with `argv` laid out as Rust expects, 4
  bytes per pointer (an operating system's argument block may differ:
  trylobite's holds words, so its start code makes a copy);
- `memcpy`, `memmove`, `memset`, `memcmp` and `bcmp` (LLVM calls them
  for copies and comparisons), `strlen` (LLVM recognises the loop), and
  `abort`, for panics;
- whatever `extern "C"` functions the program declares, such as system
  calls.

Each takes and returns its values as the translated code holds them
(Binary integers, below): an `i32` result such as `memcmp`'s that is
below 0 is returned as its unsigned value, `2^32` more, or code that
tests its sign bit (`lshr` by 31, as core's sort does) reads it wrong.

`rust/tc/tests/ll/bare.s` is such a runtime for the bare machine.

## Binary integers on a ternary machine

| Rust and LLVM | detrytus |
|---------------|----------|
| a byte (`u8`, `i8`, `bool`) in memory | one tryte holding 0..255, as devices use (isa.md 11) |
| `u16`, `u32` in memory | 2 or 4 trytes, low byte first, each 0..255 |
| a pointer in memory | 4 trytes, low byte first, each 0..255 (with `--signed-pointers`, the address in two's complement: Pointers) |
| an N-bit integer in a register | a word holding its unsigned value, 0 .. 2ᴺ − 1 |
| a 33- to 256-bit integer in registers | a word per 32 bits, low first (Wide integers) |
| `i1` (a comparison) | 0 or 1 |
| a pointer in a register | the address |

LLVM's integers have no sign of their own, so one form serves both:
the unsigned value. An operation that can leave the range (`add`,
`sub`, `mul`, `shl`, `trunc`) wraps back with `band` against 2ᴺ − 1,
unless LLVM marks it `nuw`, which promises it does not wrap. The
binary operations of isa.md 7.6 do the bitwise work: `and`, `or` and
`xor` are `band`, `bor` and `bxor`, and the shifts are `bsh`. Unsigned
division, remainder and comparison work on the stored value directly.
For the signed ones (`sdiv`, `srem`, `ashr`, `slt` and so on) a value
is first made signed, `(x ⊕ 2ᴺ⁻¹) − 2ᴺ⁻¹`, and the result wrapped
again.

A 32-bit product can reach 2⁶⁴, beyond a word (about 2⁴¹·⁸), so `mul`
is done in 16-bit halves unless one side is a constant small enough, or
the type is at most 20 bits, for the product to fit. A comparison that
a branch uses becomes a compare-and-branch; one kept as a value is
`cmp` followed by a `tlut` that maps its −1/0/+1 to 0 or 1.

A load of a 2- or 4-byte value calls a small helper that reads its
trytes and joins them; a store calls one that splits it (see Size).
A load or store touches the integer's bits rounded up to bytes, LLVM's
store size, and no more: an `i24`, which SROA makes of the bytes beside
an enum's tag, is 3 bytes, though it takes 4 in an array.
A struct or array value is held as one register per scalar in it (a
slice is two: pointer and length).

A shift left first drops the bits that would go past the type,
`(x mod 2^(N − k)) << k`, since a word holds only about 42 bits and
`bsh` would otherwise wrap in ternary.

### Wide integers

An integer wider than 32 bits does not fit a word, so it is held in
limbs of a word each, low first: 32 bits apiece, the last holding what
is left, each as its unsigned value. A `u64` is two limbs, a `u128`
four. In memory it is the bytes LLVM expects, the limbs one after the
other. Everywhere values are only moved (phis, selects, loads and
stores, arguments, results, struct fields) a wide integer is just its
limbs, as a slice is two scalars. The operations are worked out on the
limbs:

- `add` and `sub` carry or borrow from one limb to the next, found by
  comparing the limb's result with 2³² − 1 or with 0;
- `and`, `or`, `xor` work on each limb;
- shifts by a constant move whole limbs and then bits across them,
  masking a limb first so that it never grows past what a word holds;
  `ashr` takes the last limb signed, so that `bsh` brings in its sign;
- `mul` works in 16-bit pieces, adding up the pieces of each column
  of the product (sixteen products of two 16-bit pieces and a carry
  still fit a word), and leaves out the pieces a constant or a
  zero-extended value has none of;
- `udiv` and `urem` by a constant below 2²⁵ go a 16-bit digit at a
  time from the top, as long division does by hand, which is what
  core's formatting needs (it divides by 10 000 and by 100);
- the rest, too big to repeat at every use, are helper functions
  written in LLVM IR (`rust/tc/src/ll/lower/wide.ll`) and lowered like
  the program, only those it uses: division by anything else a bit at
  a time, signed division, shifts by an amount known at run time, and
  multiplication of two unknown values (for 64 bits, only when a
  program does it more than once). The 128-bit shifts are made of the
  64-bit ones. A helper returns its four limbs in `a0`..`a3`, which
  only these helpers do: Rust returns a `u128` through memory
  (abi.md 2.3);
- comparisons weigh the limbs' −1/0/+1 by powers of three, the last
  limb the heaviest, which gives a word with the comparison's sign:
  `eq` and `ne` test the limbs' `bxor` instead;
- `zext`, `sext` and `trunc` add, fill or drop limbs;
- a `switch` tests each case limb by limb, low first, going on to the
  next case at the first limb that differs: LLVM makes a chain of
  comparisons of an `i64` (or of an `i63`, once it knows the sign bit)
  into one.

Widths in between (33 to 63 bits, 65 to 127, up to 256) get the
operations done on the limbs, not the helpers. Core formats a `u128`
by multiplying it as a 256-bit integer by a constant and shifting,
which is how it divides by 10¹⁹, so 256 bits are needed too.

### Pointers

A pointer is an unsigned 32-bit number, as LLVM sees it, which a
program running above address 0 needs no more of: its addresses, and
the bytes that stand for them, are the same numbers. A kernel runs
below 0 too (trylobite's code near −68 900, its device pages from −9 to
−1 pages), so `--signed-pointers` makes pointers signed: in a register
a pointer is the address itself, below 0 or above, reaching ±2³¹
trytes balanced around 0 as the machine's addresses are. Where it
meets memory or an integer it becomes its 32 bits in two's complement
and back:

- a pointer is stored and loaded through helpers of its own (`ll$stp`,
  `ll$ldp`): a negative address is stored as 2³² more, and the top
  byte is read back as signed, so the call site costs what a `u32`'s
  does;
- `ptrtoint` gives the 32 bits (2³² more when the address is below 0),
  `inttoptr` takes the signed value of the low 32;
- a pointer in static data is those bytes, worked out from the label
  by the assembler; a number made into a pointer is its own bytes.

Comparisons of pointers compare the addresses. That is LLVM's
unsigned order wherever both sit on the same side of 0, which holds
for any two addresses in one object as long as no object straddles
address 0; page 0 is never mapped for that and other reasons. Copies
(`memcpy`) and arithmetic (`getelementptr`) are unchanged: one moves
bytes, the other adds to the address.

It costs time, not room: on trylobite's sort benchmark, whose programs
load pointers all the time, 10% more steps (810 to 892 million) and
about 2% more code. So it is a switch, for the code that needs it.

## Calls and frames

Calls follow abi.md: each scalar argument takes one of `a0`..`a5`, a
slice or a 64-bit integer two, and words after the sixth go on the
stack, the seventh at `0(sp)` (abi.md 2.2): core's formatting calls
`pad_integral` with eight. A result of one or two scalars comes back in
`a0` and `a1`. A call through a function pointer (a trait object's vtable, a
`fn` value) is `jalr ra`. An `alloca` gets room in the frame, below
`s0`. Static data is emitted byte by byte; a pointer in it is four
trytes worked out from the label by the assembler. An `alias` is
another name for what it aliases.

### Alignment

Some of core relies on alignment: `fmt::Arguments` tells a pointer to
its arguments from the length of a plain string by the pointer's low
bit, which is 0 only because the arguments are aligned to 4. So what
LLVM asks to align is aligned, in the binary sense: the address is a
multiple of the alignment. Static data gets `.balign N`, which the
assembler turns into padding (assembler.md). An `alloca` with an
alignment of 2 or more is placed against a base register made at the
function's start, `s0` plus an offset rounded down with `band` against
−A, since the frame pointer itself may be anywhere; the frame has A − 1
trytes to spare for it.

## Floating point

`f32` and `f64` are IEEE 754 binary floats, which run on the binary
floating point coprocessor, a device on page −15 (`docs/platform.md`);
the core's own floats are ternary (isa.md 7.11) and do not stand in
for them. A `float` is held as the integer of its bits, one word, a
`double` as a 64-bit integer, two; in memory both are the IEEE 754
bytes. Those are the words the coprocessor takes, so an operation is
its operands stored into A and B, the operation into OP, and the
result loaded from R. When the IR is read (`ir.rs`, `expand_float`),
each floating-point operation becomes such an `Op::Fpu`, or calls and
integer operations where the coprocessor has no operation:

- `fadd`, `fsub`, `fmul`, `fdiv` are `FPU_ADD` and the rest; `frem`
  calls `fmod`;
- `fcmp` is one `FPU_CMP`, whose answer the lowering turns into 0
  (less), 1 (equal), 2 (greater) or 3 (unordered), and each predicate
  one or two integer comparisons of that;
- `fptosi`, `fptoui`, `sitofp`, `uitofp` to and from 32 or 64 bits are
  `FPU_TO_I32`, `FPU_FROM_U64` and the rest, extending or truncating
  narrower integers; 128 bits call `__fixdfti` and kin; `fpext` and
  `fptrunc` are `FPU_CVT`;
- Rust's `as` casts are `llvm.fptosi.sat` and `llvm.fptoui.sat`; the
  coprocessor's conversions (and `__fixdfti` and kin for 128 bits)
  already saturate and turn a NaN into 0, so 32 bits and wider is one
  operation, and narrower ones are clamped after;
- `fneg`, `llvm.fabs` and `llvm.copysign` work on the sign bit;
- `llvm.sqrt`, `fma`, `fmuladd`, `minnum`, `maxnum`, `minimumnum` and
  `maximumnum` are coprocessor operations; `floor`, `ceil`, `trunc`,
  `round`, `rint`, `roundeven`, `nearbyint`, `minimum` and `maximum`
  call library functions (`floorf`, `fminimum`, ...).

The library functions are our own, in `rust/tc/src/ll/lower/float.ll`,
written in LLVM IR as `lower/wide.ll` is, and translated with the
program; only what the program calls is lowered, so a program without
floating point pays nothing. They do their arithmetic on the
coprocessor: rounding to an integer cuts fraction bits off, or adds and
takes away 2^52 for ties to even; `fmod` is long division of the
significands, which is exact; conversions from 128 bits convert the top
64 bits with a sticky bit and scale by a power of two, so they round
once. The `float` versions widen to `double` and narrow the answer,
which is exact for all but those conversions, which have their own.
`rust/tc/tests/float_lib.rs` runs each against the host's answers.

Floating point brought in what core's float code needs: `llvm.ctlz`,
`llvm.umin` and the like on 64- and 128-bit integers, `llvm.ctpop`,
`llvm.fshl`/`fshr` (funnel shifts) and `llvm.ucmp`/`scmp`. Core's
formatting of floats (`{}`, `{:.3}`, `{:e}`) works: Grisu with Dragon
as its fallback, as on any target. It is not small: most of
`rust/tc/tests/ll/floats.rs` is float formatting.

## Atomics

The target, `riscv32imac`, has atomics up to 32 bits, so core's
`AtomicU32`, `AtomicBool` and the rest come out as LLVM's `load atomic`,
`store atomic`, `atomicrmw` and `cmpxchg`. Bytes in memory are trytes,
four to a `u32`, while detrytus's atomic instructions work on whole
words (isa.md 7.9), so an atomic `u32` cannot be one `amoadd`. Instead
the program has one lock, a word `ll$held`, and every atomic
operation becomes its plain operations between taking it and giving it
back: `atomicrmw max` is a load, a comparison, a select and a store,
`cmpxchg` a load, an `eq`, a select and a store. Taking the lock is a
call to `ll$lock`, which spins on `amoswap.aq` until the word was 0;
`ll$unlock` puts 0 back with `amoswap.rl`. Both are `jal t6` helpers
that change only `t2`..`t6` (`hcall` in alloc.rs), as the load and
store helpers do. A `fence` becomes nothing, since a hart sees its own
accesses in order and the lock orders those of others.

So atomic operations are atomic against each other on any number of
harts, which is what Rust promises; a plain access racing an atomic
one is undefined behaviour in Rust anyway. A thread preempted while it
holds the lock makes others spin until it runs again, which is slow but
correct. `atomicrmw` on floats and the wrapping increments
(`uinc_wrap`) are refused.

## Words and the machine

A kernel handles what no user program does: whole ternary words
(saved registers, system-call arguments, page-table entries, messages),
CSRs, and the instructions only ring 0 runs. Rust reaches them through
builtins, functions named `detrytus.*` that the translator turns into
the instructions in place instead of calls. Rust declares them with
`#[link_name]`:

```rust
unsafe extern "C" {
    #[link_name = "detrytus.lw"]
    fn lw(p: *const u8) -> i64;
}
```

A word is an `i64` on the Rust side: its value, which a word holds up to
±(3²⁷ − 1)/2, about ±3.8 × 10¹², so an `i64` holds every one exactly and
Rust's own arithmetic and comparisons work on it. The translator joins
the two limbs into one word on the way in (`mul`, `add`) and splits a
word into limbs on the way out (`band`, `sub`, `div`); a value outside
a word's range does not survive the trip.

| Builtin | Does |
|---------|------|
| `lw(p) -> i64`, `sw(p, w)` | the aligned word at `p` (a misaligned one traps) |
| `csrr(csr) -> i64`, `csrw(csr, w)` | a CSR, by number (isa.md 8.2); `csr` must be a constant |
| `tex(w, pos, len) -> i64`, `tins(w, v, pos, len) -> i64` | a trit field of `w`, and `w` with it replaced by `v` (isa.md 7.3); `pos` and `len` constants, `0 ≤ pos`, `1 ≤ len`, `pos + len ≤ 27` |
| `fence.vm(addr, asid)` | flush translations; a null `addr` or a 0 `asid` means all (isa.md 9.7) |
| `wfi()` | wait for an interrupt |

Each is a call to Rust, so it is never left out or moved past other
memory accesses: `lw` and `sw` serve device registers as well as
memory. `tret` and trap entry have no builtin: they belong in the
assembly stub around the Rust code, which saves and restores every
register.

## Size

Faithful bytes cost room: a `u32` in memory is four trytes to read
and join, and every wrap is a `band`. Memory has to stay a byte a
tryte, since LLVM reads integers as bytes and bytes as integers
(`memcpy` of a struct, a `u64` read as two `u32`s), so a value cannot
simply live in a word there. What helps instead:

- **Helpers for loads and stores.** A 2- or 4-byte load or store is
  a call, `jal t6`, to one of four helpers (`ll$ld2`, `ll$ld4`,
  `ll$st2`, `ll$st4`) emitted once, with the address in `t5` and the
  value in `t4`. They change only `t2`..`t6`, which the register
  allocator knows (`hload` and `hstore` in alloc.rs), so values in
  other registers stay. Three instructions replace up to eleven.
- **Large constants made once.** A constant beyond `addi`'s range
  costs two instructions; one a function makes four times or more
  (mostly 2³² − 1, the mask that wraps 32-bit results) is made once at
  its start.
- **Folding.** The signed form of a constant is worked out at once,
  and 64-bit operations leave out halves known to be zero.

On trylobite, a Rust hello is 367 trytes, its runtime included, an
echo written with iterators 596 (671 before these), against 147 for
the Trice echo, and `rfact`, which prints a table of 64-bit factorials
with `println!`, 3 960 (5 715 before). `rust/tc/tests/ll/fmt.rs`,
which formats integers, strings, padding and `Debug`, is about 14 000
trytes on the bare machine.

## Not yet

The translator refuses these with an error that names the function and
the line:

- integers wider than 256 bits; for widths other than 64 and 128,
  shifts by a variable amount and division by anything but a constant
  below 2²⁵;
- `half`, `fp128` and the other floating-point types, vectors,
  varargs;
- results of more than four words;
- variable-sized `alloca`;
- `llvm.cttz`; `llvm.smul.with.overflow` and the overflow intrinsics,
  `llvm.bswap` and `llvm.bitreverse` on integers wider than a word
  (those of up to 32 bits are done: `umul`, `uadd`, `usub`, `sadd`,
  `ssub`).
