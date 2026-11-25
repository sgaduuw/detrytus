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
allows on a stable toolchain), so that its IR is there too. Panics
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
how the tests keep only what they need of core.

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

`rust/tc/tests/ll/bare.s` is such a runtime for the bare machine.

## Binary integers on a ternary machine

| Rust and LLVM | detrytus |
|---------------|----------|
| a byte (`u8`, `i8`, `bool`) in memory | one tryte holding 0..255, as devices use (isa.md 11) |
| `u16`, `u32`, a pointer in memory | 2 or 4 trytes, low byte first, each 0..255 |
| an N-bit integer in a register | a word holding its unsigned value, 0 .. 2ᴺ − 1 |
| a 33- to 64-bit integer in registers | two words: the low 32 bits, then the rest |
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

A load of a wider value reads its trytes and joins them with `bsh` and
`bor`; a store splits it with `band` and `bsh`.

### 64-bit integers

A `u64` or `i64` does not fit a word, so it is held as two, low half
first, each a 32-bit unsigned value; in memory it is the eight bytes
LLVM expects, the two halves one after the other. Everywhere values are
only moved (phis, selects, loads and stores, arguments, results,
struct fields) a 64-bit integer is just two scalars, as a slice is.
The operations are worked out on the halves:

- `add` and `sub` carry or borrow between them, found by comparing the
  low result with 2³² − 1 or with 0;
- `and`, `or`, `xor` work on each half;
- shifts by a constant move bits across the halves, masking a half
  first so that it never grows past what a word holds;
- `mul` works in 16-bit pieces, adding up the pieces of each column
  of the product (four products of two 16-bit pieces and a carry still
  fit a word), and leaves out the pieces a constant or a zero-extended
  value has none of;
- `udiv` and `urem` by a constant below 2²⁶ go a 16-bit digit at a
  time from the top, as long division does by hand, which is what
  core's formatting needs (it divides by 10 000 and by 100);
- comparisons weigh the halves' −1/0/+1 three to one, which gives a
  word with the comparison's sign: `eq` and `ne` test the halves'
  `bxor` instead;
- `zext`, `sext` and `trunc` take or make the high half.

Shifts by a variable amount, other divisions and 128-bit integers are
refused for now. A struct or array
value is held as one register per scalar in it (a slice is two:
pointer and length).

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

## Size

Faithful bytes cost room: a `u32` loaded from memory is four tryte
loads joined together, and every wrap is a `band`. On trylobite, a
Rust hello is 370 trytes, its runtime included, and an echo written
with iterators 671, against 147 for the Trice echo. `core::fmt` costs
more: `rust/tc/tests/ll/fmt.rs`, which formats integers, strings,
padding and `Debug`, is about 21 000 trytes on the bare machine. Keeping 32-bit values in one word where that is
safe is the obvious next saving.

## Not yet

The translator refuses these with an error that names the function and
the line:

- 128-bit integers; for 64-bit ones, shifts by a variable amount,
  signed division and division by anything but a constant below 2²⁶;
- floating point, atomics, vectors, varargs;
- results of more than two words;
- variable-sized `alloca`;
- the intrinsics for counting bits other than `llvm.ctlz`, swapping
  bytes and rotating (`llvm.ctpop`, `llvm.cttz`, `llvm.bswap`,
  `llvm.fshl` and others).
