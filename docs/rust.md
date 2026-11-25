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
how the tests keep only what they need of core; `--root NAME` (any
number of times) prunes from those functions instead of `main`.

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
- `zext`, `sext` and `trunc` add, fill or drop limbs.

Widths in between (33 to 63 bits, 65 to 127, up to 256) get the
operations done on the limbs, not the helpers. Core formats a `u128`
by multiplying it as a 256-bit integer by a constant and shifting,
which is how it divides by 10¹⁹, so 256 bits are needed too.

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

detrytus has no floating-point hardware yet (isa.md specifies the
instructions, the emulator does not run them), so `f32` and `f64` are
done in software, the way rustc does it on a RISC-V core without an
FPU: by calling the functions of Rust's own `compiler_builtins`. A
`float` is held as the integer of its bits, one word, a `double` as a
64-bit integer, two; in memory both are the IEEE 754 bytes. When the IR
is read (`ir.rs`, `expand_float`), each floating-point operation becomes
calls and integer operations:

- `fadd`, `fsub`, `fmul`, `fdiv` call `__adddf3` and the rest (`sf`
  for `float`), `frem` calls `fmod`;
- `fcmp` calls one of `__eqdf2`, `__ltdf2`, `__gedf2`, `__unorddf2`
  and so on and compares the answer with 0, as LLVM itself does for a
  target without an FPU; `ueq` and `one` take two calls;
- `fptosi`, `fptoui`, `sitofp`, `uitofp` call `__fixdfsi`,
  `__floatsidf` and the rest for 32, 64 or 128 bits, extending or
  truncating narrower integers; `fpext` and `fptrunc` call
  `__extendsfdf2` and `__truncdfsf2`;
- Rust's `as` casts are `llvm.fptosi.sat` and `llvm.fptoui.sat`.
  compiler_builtins' conversions already saturate and turn a NaN into
  0, so 32 bits and wider is one call; narrower ones are clamped after;
- `fneg`, `llvm.fabs` and `llvm.copysign` work on the sign bit;
- `llvm.sqrt`, `floor`, `ceil`, `trunc`, `round`, `rint`, `fma`,
  `minnum`, `maxnum` and their kin call compiler_builtins' libm
  (`sqrt`, `floorf`, `fmin`, ...).

The functions come from `rust/tc/src/ll/lower/float.ll`: the part of
compiler_builtins (and the bits of core it uses) that those calls
reach, pruned from the IR that rustc emits when it builds core, by
`lower/float.sh`. It is translated with the program; only what the
program calls is lowered, so a program without floating point pays
nothing. compiler_builtins is MIT and Apache-2.0 licensed (its libm
part comes from musl, MIT), as the file's header says.

Floating point brought in what these functions need: `llvm.ctlz`,
`llvm.umin` and the like on 64- and 128-bit integers, `llvm.ctpop`,
`llvm.fshl`/`fshr` (funnel shifts) and `llvm.ucmp`/`scmp`. Core's
formatting of floats (`{}`, `{:.3}`, `{:e}`) works: Grisu with Dragon
as its fallback, as on any target. It is not small:
`rust/tc/tests/ll/floats.rs` is about 39 000 trytes, most of it float
formatting.

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
- `llvm.cttz`, `llvm.bswap`, `llvm.bitreverse` and the overflow
  intrinsics (`llvm.umul.with.overflow` and others).
