# Trice

Version 0.1, **draft**. It will change as the compiler meets real
programs. The first compiler (`rust/tc`) covers this page except where
section 12 says otherwise.

Trice is a small systems language for detrytus: C's statements and
expressions over the machine's own types. Everything is signed, a
boolean is a trit with a third value for "unknown", and C's bitwise
operators are trit-wise ones. It is the first compiled language for
the machine, written to bootstrap the tools of
[trylobite](https://github.com/sgaduuw/trylobite) and to leave room for
a Rust backend later. Its compiler is `detrytus tc`, sources end in
`.tc`, and compiled code follows the ABI in [`abi.md`](abi.md).
CONTEXT.md (2026-11-23) records why the language is shaped this way.

## 1. A program

```c
import "../include/layout.s";      // constants: SYS_WRITE, STDOUT, ...
import "../lib/start.s";           // trylobite's program start

int write(int fd, tryte *buf, int n) {
    return sys(SYS_WRITE, fd, buf, n);
}

int strlen(tryte *s) {
    int n = 0;
    while (s[n] != 0) n++;
    return n;
}

int main(int argc, tryte **argv) {
    int failed = 0;
    for (int i = 1; i < argc; i++) {
        if (write(STDOUT, argv[i], strlen(argv[i])) < 0) failed = 1;
        if (i + 1 < argc) write(STDOUT, " ", 1);
    }
    write(STDOUT, "\n", 1);
    return failed;
}
```

`detrytus tc echo.tc -o echo.s` writes one assembly file;
`detrytus asm echo.s -o echo.dti` assembles it.

## 2. Lexical structure

- Comments: `//` to the end of the line, and `/* ... */`.
- Identifiers: a letter or `_`, then letters, digits and `_`. They may
  not be register names (`a0`, `sp`, `r-1` is not an identifier anyway)
  or keywords.
- Integer literals: decimal `42`, hexadecimal `0x2a`, balanced ternary
  `0t+0-` (8), as in the assembler. A literal must fit a word.
- Character literals: `'a'`, `'\n'`, the code point, at most 9,841.
- String literals: `"..."`, a tryte array of code points ending in 0,
  as the assembler's `.string`. Escapes are the assembler's: `\n`,
  `\t`, `\0`, `\\`, `\'`, `\"`; any other is an error.
- Keywords: `bool break const continue else extern false for if import
  int return sizeof struct trit true tryte unknown void which while`.

## 3. Types

| Type | Size (trytes) | Values |
|------|---------------|--------|
| `trit` | 1 | −1, 0, +1 |
| `bool` | 1 | `true` (+1), `false` (−1), `unknown` (0) |
| `tryte` | 1 | ±9,841; also a code point |
| `int` | 3 | ±3,812,798,742,493 |
| `T*` | 3 | an address; `p + 1` steps by `sizeof(T)` |
| `T[n]` | `n × sizeof(T)` | |
| `struct S` | as laid out by abi.md 1.2 | |
| `void` | | function results only |

There are no unsigned types, no `long` or `short`, and no floats yet.

**Arithmetic is done on words and wraps.** A `trit`, `bool` or `tryte`
operand becomes an `int` of the same value; the result of `+ - * / %`
is an `int`, reduced into the word range as the ISA's `wrap` does, so
overflow is defined. Assigning an `int` to a `tryte` keeps its low 9
trits. Because balanced values need no sign or zero extension, nothing
else happens on the way.

**Conversions** between types happen only where nothing can be lost
(`tryte` or `trit` to `int`, `T[n]` to `T*`). A cast does the rest:
`(tryte) x` narrows, `(int) p` and `(T*) x` convert between pointers
and integers, `(trit) b` and `(bool) t` reinterpret the three values.
There is no conversion from `int` to `bool`: write a comparison.

`sizeof(T)` and `sizeof expr` count trytes.

## 4. Truth values

`bool` has three values. Comparisons give `true` or `false`; `unknown`
comes from the literal, from a function, or from memory not yet written
(memory starts as 0, so a fresh flag is `unknown`). It is meant for
"not decided" and "not done yet".

The logical operators follow Kleene's three-valued logic, which the ISA
computes in one instruction each:

| `a` | `b` | `a && b` (`tmin`) | `a \|\| b` (`tmax`) | `!a` |
|-----|-----|------|------|------|
| `true` | `unknown` | `unknown` | `true` | `false` |
| `false` | `unknown` | `false` | `unknown` | `true` |
| `unknown` | `unknown` | `unknown` | `unknown` | `unknown` |

`&&` skips its right side when the left is `false`, and `||` when it is
`true`, as in C. `&` and `|` on booleans are the same operations without
skipping.

**Conditions** in `if`, `while`, `for` and `?:` must be `bool`;
`if (n)` on an `int` is an error, write `if (n != 0)`. They take their
branch only on `true`: `unknown`, like `false`, goes to `else` or ends
the loop, as SQL's `WHERE` drops rows whose condition is `NULL`. Code
that must tell the three apart uses `which` (section 7).

## 5. Expressions

From tightest to loosest; all binary operators group left to right
except assignment and `?:`.

| Operators | Meaning |
|-----------|---------|
| `f(x)` `a[i]` `s.f` `p->f` `x++` `x--` | call, index, field, post-increment |
| `-x` `!b` `*p` `&x` `++x` `--x` `(T) x` `sizeof` | negate, not, load, address, cast |
| `*` `/` `%` | multiply; divide and remainder truncating toward zero (`div`, `rem`) |
| `+` `-` | add, subtract; pointer plus or minus an integer |
| `<<` `>>` | shift by trits: `x << n` is `x × 3ⁿ`, `x >> n` is `x / 3ⁿ` rounded to nearest (`sh`) |
| `<=>` | three-way compare: a `trit`, −1, 0 or +1 (`cmp`) |
| `<` `<=` `>` `>=` | ordering, giving `bool` |
| `==` `!=` | equality, giving `bool` |
| `&` | tritwise minimum: Kleene AND on `bool`, `tmin` on every trit of an `int` |
| `\|` | tritwise maximum: Kleene OR on `bool`, `tmax` on every trit of an `int` |
| `&&` | Kleene AND, short-circuit on `false` |
| `\|\|` | Kleene OR, short-circuit on `true` |
| `c ? a : b` | `a` when `c` is `true`, else `b` |
| `=` `+=` `-=` `*=` `/=` `%=` `<<=` `>>=` `&=` `\|=` | assignment |

There is no `~`: negation already flips every trit, so `-x` is the
only one. There is no `^`: use `tsum` or `tmul` (section 8). `!` takes a
`bool` and `-` a number; both are the same instruction.

Comparisons take two numbers, or two pointers of the same type.

## 6. Declarations

```c
const int NBUF = 243;              // a compile-time constant
int count;                         // a global, 0 at start
tryte buf[NBUF];                   // a global array
struct entry { int inode; tryte name[27]; };

int add(int a, int b) { return a + b; }
extern int wait2(int pid);         // defined in assembly
```

Globals start as 0, or as their initialiser, which must be a constant
(an integer, a string, the address of a global or of an element of a
global array, or a list of them in braces). Locals are declared
anywhere in a block and live to its end; an uninitialised local holds
whatever its stack slot held, and a local's initialiser may be any
expression.

```c
int primes[5] = { 2, 3, 5, 7 };          // primes[4] is 0
tryte word[8] = "ternary";               // a string fills a tryte array, 0 included
struct entry table[2] = { { 1, "one" }, { 2, "two" } };
```

A list in braces initialises an array element by element or a struct
field by field, in order, and what it leaves out is 0. A struct is
assigned, and initialised from another, by copying it; it is passed to
and returned from functions by pointer.

`import "path";` brings in another file, found relative to the one that
imports it, once however many times it is named:

- a `.tc` file: its declarations, compiled into this program;
- a `.s` file: included in the output where the import stands, and
  every `.equ` in it becomes a constant of the same name. This is how a
  program uses `include/layout.s` without copying a number, and how
  OS-specific start code arrives.

A program is compiled whole: the files it imports are its library. Or
each file is compiled on its own into an object, and the objects are
linked (section 13).

## 7. Statements

`if`/`else`, `while`, `for`, `break`, `continue`, `return`, blocks and
expression statements are C's.

`which` branches three ways with one `br3`:

```c
which (key <=> a[mid]) {           // a trit or an int: by its sign
    < : hi = mid - 1;
    = : return mid;
    > : lo = mid + 1;
}

which (done) {                     // a bool
    true:    report();
    unknown: retry();
    false:   give_up();
}
```

An arm may be left out, and then nothing happens for that case. Arms do
not fall through into one another.

`asm { ... }` places its lines in the output as they are. The compiler
knows nothing about what they do, so they may rely only on the ABI: an
`asm` block is for code with no Trice equivalent, best kept to whole
functions in an imported `.s` file.

## 8. Builtins

Each builtin is one instruction, or a few, and is named after it, so
[`isa.md`](isa.md) is its manual.

| Builtin | Gives |
|---------|-------|
| `abs(x)`, `min(a, b)`, `max(a, b)` | `abs`, `min`, `max` |
| `divr(a, b)`, `remr(a, b)` | division rounded to nearest, and its remainder (7.1) |
| `tmul(a, b)`, `tsum(a, b)`, `tcons(a, b)`, `tany(a, b)`, `tabs(x)` | the named truth tables (7.5) |
| `tlut(a, b, T)` | any two-input trit function; `T` a constant nine-trit table, −9,841 to 9,841 (7.5) |
| `tex(x, pos, len)`, `tins(x, v, pos, len)` | a trit field, and `x` with one replaced; `pos` and `len` constant, `0 ≤ pos`, `1 ≤ len`, `pos + len ≤ 27` (7.3) |
| `lzt(x)`, `tzt(x)`, `nzt(x)` | trit counts (7.4) |
| `band(a, b)`, `bor(a, b)`, `bxor(a, b)`, `bsh(a, n)` | binary operations for binary data (7.6) |
| `sys(a0, a1, a2, a3, a4, a5)` | puts its arguments, up to six, in `a0` onward, runs `sys`, gives `a0` |

`sys` carries no meaning of its own: what the registers mean is the
operating system's. On trylobite `a0` is the call number and the result
comes back in `a0`. The compiler takes it that a system call changes
only `a0` to `a5` and leaves every other register as it was, as a trap
handler that can resume a preempted program must.

A `tlut` table reads best as a `0t` literal, entry 8 first, as in
isa.md's table: `tlut(a, b, 0t+0-00----)` is `a & b`. `tins` gives a
new value and leaves `x` as it was, so packing a word is a chain:

```c
int pack(int hi, int lo) { return tins(tins(0, lo, 0, 9), hi, 9, 9); }
int hi_of(int w)         { return tex(w, 9, 9); }
```

## 9. Diagnostics

Besides errors, the compiler warns about comparing with the literal
−1 (`x == -1`, `x < -1`): on this machine −1 is an ordinary value, and a
"no value" result is tested by sign or compared with a named constant.

## 10. Output

The compiler writes one assembly file: the imports, then each function,
then the globals, under `.compress` so the assembler bundles what it
can. A function that nothing reaches from `main` is checked and left
out; a function an `asm` block or imported assembly names counts as
reached. So is one copied into every caller (section 12), and a global
or string that no code left in names. Where the code goes is the start code's business: on trylobite,
`lib/start.s` opens the program's section at `V_USER`, says where `gp`
points with `.gp`, and calls `main(argc, argv)` before exiting with its
result. Each statement's code carries a comment naming its source line,
so listings and the debugger lead back to the source.

## 11. Not yet

Floats, `switch`, function pointers, struct values in calls and
results, varargs and a preprocessor. Each can be
added without changing what is above.

## 12. The first compiler

`detrytus tc FILE` prints the assembly; `-o OUT.s` writes it, and
`-o OUT.dti` assembles it into an image as well. With `-c` it compiles
one file on its own (section 13). Errors name the file and line;
warnings go to standard error.

Code generation walks the tree, with every value in a virtual
register of its own; locals whose address is never taken are virtual
registers too, and the rest live in the frame. A back end then works on
each function as assembly over those registers:

- **Redundancy.** Within a block, a constant, an address, the same
  arithmetic on the same registers, or a global already loaded (or just
  stored) is reused rather than computed again. Not across a call,
  where keeping it would cost a saved register, and never a load
  through a pointer, which may be a device's register. Copies of a
  value written once are read through.
- **Dead code.** An instruction whose result nothing reads goes, by
  liveness over the function's control flow.
- **Registers.** An interference graph is coloured with `t0` to `t6`,
  then `a5` to `a0`, then `s1` to `s8`, so a saved register (which
  costs a save) holds only what lives across a call. The two sides of a
  copy are merged first when that cannot force a spill, and a result
  prefers its operand's register, since `rd = rd op rs` has a
  compressed form and `rd = rs1 op rs2` none. What does not fit goes to
  the frame, and allocation runs again.
- **Copying small functions in.** A function that calls nothing, has
  nothing in memory and no `asm` is copied into its callers when it has
  at most four instructions besides copies, or only one caller. A
  function every caller copied is left out.
- **Clean-up.** A branch over a jump branches the other way, a jump to
  a jump goes on to the end, code nothing reaches goes, and so do
  labels nothing jumps to, which would keep their neighbours out of a
  bundle.
- **Frames.** A function with nothing to keep has no frame; one that
  calls, or uses saved registers, keeps them from `sp`; only one with
  memory locals or more than six parameters keeps a frame pointer.

Sizes of trylobite's programs, in trytes, the buffers included:

| Program | Hand-written | Tree walker | With the back end |
|---------|-------------:|------------:|------------------:|
| `wc` | 659 | 1,111 | 901 |
| `grep` | 1,406 | 1,831 | 1,663 |
| `head` | 629 | 1,025 | 851 |
| `upper` | 312 | 459 | 375 |
| `echo` | 70 | 220 | 154 |

Most of what remains is structure the hand-written programs leave out:
the start code, layers of small library functions, and the saves a
function owes its caller.

A `which` arm must be within `br3`'s reach of 3,280 instructions.

## 13. Compiling files separately

`detrytus tc -c FILE.tc` compiles one file on its own, for the linker
([`linker.md`](linker.md)): `-o FILE.dto` writes an object, `-o
FILE.s` the assembly. `detrytus link` takes `.tc` files too and
compiles each that way, so a program and its library build as

```sh
detrytus link bin/echo.tc lib/io.tc -o echo.dti
```

Compiled on its own, a file differs from a whole program in three ways:

- **What it imports from other `.tc` files is only declared.** Their
  functions, globals, constants and structs can be used, but their code
  and data belong to their own objects. Their own imports count only
  for their constants: an assembly file is included only by the file
  that imports it directly.
- **Every function and global it defines is `.global`**, so other
  objects can use it, and each is in a part of its own (`.part`,
  linker.md), so the linker leaves out the ones no kept code uses. A
  small function is still copied into its callers in the file, and kept
  for the others.
- **Its code is in the section the assembly puts it in**: after a start
  file's `.section`, where that leaves off, and otherwise in `text`,
  which the linker places after the sections before it. On trylobite,
  a program's file imports `lib/start.s`, which places it at `V_USER`,
  and a library's code follows it.

A global the file reaches through `gp` (section 10) is one instruction
whichever object holds it, but the linker finishes its offset, so that
instruction is never put in a compressed bundle (linker.md). So are
calls between functions, which are in different parts. Linked programs
are larger than whole ones for that reason.
