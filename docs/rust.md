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
deps=target/riscv32imac-unknown-none-elf/release/deps
detrytus ll --include start.s $deps/prog-*.ll $deps/core-*.ll -o prog.s
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
`bor`; a store splits it with `band` and `bsh`. A struct or array
value is held as one register per scalar in it (a slice is two:
pointer and length).

## Calls and frames

Calls follow abi.md: each scalar argument takes one of `a0`..`a5`, a
slice two, and a result of one or two scalars comes back in `a0` and
`a1`. A call through a function pointer (a trait object's vtable, a
`fn` value) is `jalr ra`. An `alloca` gets room in the frame, below
`s0`. Static data is emitted byte by byte; a pointer in it is four
trytes worked out from the label by the assembler.

## Size

Faithful bytes cost room: a `u32` loaded from memory is four tryte
loads joined together, and every wrap is a `band`. On trylobite, a
Rust hello is 370 trytes, its runtime included, and an echo written
with iterators 671, against 147 for the Trice echo. Keeping 32-bit
values in one word where that is safe is the obvious next saving.

## Not yet

The translator refuses these with an error that names the function and
the line:

- integers wider than 32 bits (`u64`, `u128`), which need two words;
  core's formatting uses them, so `core::fmt` does not work yet;
- floating point, atomics, vectors, varargs;
- calls with more than six argument words, results of more than two;
- variable-sized `alloca`;
- the intrinsics for counting bits, swapping bytes and rotating
  (`llvm.ctpop`, `llvm.ctlz`, `llvm.bswap`, `llvm.fshl` and others).

Static data is not aligned to 4: the assembler aligns to powers of 3
only. Nothing in core depends on it for correctness.
