# detrytus ABI

Version 0.1, **draft**. How compiled code lays out data and calls
functions, so that code from different compilers (Trice first,
[`trice.md`](trice.md); a Rust backend later) and hand-written assembly
call each other with no glue. It builds on the register roles of
[`isa.md`](isa.md) 4.2 and settles what 4.2 leaves open. It says
nothing about any operating system: how a program starts and how it
makes system calls belongs to the OS (for trylobite, its
`docs/kernel.md`).

## 1. Data

### 1.1 Scalar types

| Type | Size | Alignment | Values |
|------|------|-----------|--------|
| trit | 1 tryte | 1 | −1, 0, +1 |
| boolean | 1 tryte | 1 | +1 true, −1 false, 0 unknown (isa.md 4.2) |
| tryte | 1 tryte | 1 | ±9,841 |
| word | 3 trytes | 3 | ±H27 |
| pointer | 3 trytes | 3 | an address |
| float | 3 trytes | 3 | isa.md 7.11 |

Every integer type is signed; there are no unsigned types to extend.
A tryte loaded with `lt` keeps its value in a full register, so a value
in a register is always the value itself, whatever type it came from.
Code that narrows a register to a tryte (to store it, or to pass it as
a tryte) keeps its low 9 trits, which is `st`'s behaviour.

The null pointer is 0.

### 1.2 Centred layout

A word-aligned object is centred on its address, like every aligned
unit (isa.md 5.2). Its address is a multiple of 3 and names the centre
of its first word, so the object begins one tryte below its address. An
object with alignment 1 begins at its address.

Struct fields are placed in declaration order by the rule the assembler
uses for `.word` and `.tryte` (assembler.md, Layout), measured from the
struct's address:

1. A cursor starts at −1 for a word-aligned struct, at 0 otherwise. A
   struct is word-aligned when any field is.
2. A field of alignment 1 goes at the cursor, which then moves past it.
3. A word-aligned field goes at the first multiple of 3 whose tryte
   below is at or after the cursor, and the cursor moves past its last
   tryte.
4. The struct's size runs from its first tryte to its last, rounded up
   to a multiple of 3 when it is word-aligned, so that arrays tile.

```
struct example {        offset   trytes
    int   a;              0      -1 .. 1
    tryte b;              2       2
    tryte c;              3       3
    int   d;              6       5 .. 7     (tryte 4 is padding)
};                       size 9, word-aligned
```

An array's elements follow one another at a stride of the element's
size, and the array's address is its first element's. A struct of words
only has the familiar offsets 0, 3, 6 and so on.

## 2. Calls

### 2.1 Registers

The roles are isa.md 4.2's:

| Register | Name | Role | Kept across a call |
|----------|------|------|--------------------|
| `r1`..`r6` | `a0`..`a5` | arguments; results in `a0`, `a1` | no |
| `r7`..`r13` | `t0`..`t6` | temporaries | no |
| `r-1`..`r-9` | `s0`..`s8` | saved; `s0` the frame pointer when there is one | yes |
| `r-10` | `gp` | global pointer, set once at start | never changes |
| `r-11` | `tp` | thread pointer, reserved | never changes |
| `r-12` | `ra` | return address | no |
| `r-13` | `sp` | stack pointer | yes |

### 2.2 Arguments

The first six arguments go in `a0` to `a5`, one register each, in
order. A tryte, trit or boolean argument travels as its value in a full
register.

Arguments after the sixth go on the stack: at the call, the seventh is
the word at `0(sp)`, the eighth at `3(sp)`, and so on. The caller makes
that room and owns it.

A struct argument is passed as a pointer to a copy the caller makes;
the callee may change the copy.

### 2.3 Results

A result of one word or smaller is in `a0`. A result of two words is in
`a0` (the first) and `a1` (the second): a value and a status, or a
pointer and a length. A larger result is written to memory the caller
provides: its address is passed as a hidden first argument in `a0`,
which moves the visible arguments along by one, and comes back in `a0`.

A function that can fail and otherwise returns a value that is never
negative reports failure as a negative result, compared by sign or by a
named constant, never by a bare −1.

### 2.4 The stack

The stack grows toward negative addresses. `sp` always holds a multiple
of 3 and points at the last word pushed, so a push is `addi sp, sp, -3`
then `sw x, 0(sp)`. A function that needs a frame lowers `sp` once on
entry by a multiple of 3 and raises it by the same amount before it
returns. There is no red zone: memory below `sp` may change at any time.

A frame holds, from the top: the saved `ra` (functions that make calls),
the saved `s` registers it uses, then its locals. Placing the most used
locals nearest `sp` lets `c.lwsp` and `c.swsp` (isa.md 6.1) reach them.

A function with a frame pointer sets `s0` to the value `sp` had on
entry, and saves `ra` at `-3(s0)` and the caller's `s0` at `-6(s0)`, so
a debugger can walk the chain. Other functions need not keep one.

### 2.5 Code

A function's entry point is an aligned word. A return address may name
slot B of a bundle (isa.md 6.1), so a function returns with `jalr zero,
0(ra)` (`ret`), never by adding to `ra`.

## 3. Symbols

A program assembled from one source shares one symbol table among all
its names (assembler.md, Includes); one linked from objects shares its
global names (linker.md). A compiler
emits a function or global under its source name, so listings and the
debugger show it, and names the labels it invents with a `$`, which
assembly names may contain but source identifiers may not.

A name another object uses is made `.global`. The object format and
the linker are in [`linker.md`](linker.md); a Rust backend will use
them as well.
