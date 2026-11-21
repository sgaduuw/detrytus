# detrytus assembler

`detrytus asm FILE` prints a listing, or with `-o OUT.dti` writes an
image; `detrytus run FILE...` assembles and runs. The instruction set is in [`isa.md`](isa.md); this page covers the
source syntax.

## Lines

```asm
label:  mnemonic  operand, operand    # comment
```

Comments start with `#` or `;`. A line may carry several labels, or only
labels. Mnemonics and register names are case-insensitive; symbols are
not.

## Registers

`r-13` .. `r13`, or the ABI names from `isa.md` 4.2: `zero`, `a0`..`a5`,
`t0`..`t6`, `s0`..`s8` (`fp` is `s0`), `gp`, `tp`, `ra`, `sp`.

## Numbers and expressions

| Form | Example | Value |
|------|---------|-------|
| decimal | `42` | 42 |
| balanced ternary, most significant first | `0t+0-` | 8 |
| hexadecimal | `0x2a` | 42 |
| character | `'a'`, `'\n'` | its code point |
| symbol | `loop` | its address or `.equ` value |
| current address | `.` | address of this statement |

Operators: `+ - * / %` and parentheses, with `/` and `%` truncating
toward zero. One comparison, `== != < <= > >=`, may join two sums: it
binds loosest and gives 1 when it holds, 0 when not. In `.assert`,
`defined(NAME)` is 1 when NAME is a label or constant anywhere in the
program, 0 when not.

## Layout: words are centred

A word at address `a` occupies trytes `a-1`, `a`, `a+1` (`isa.md` 5.2).
The assembler tracks the next free tryte, and each word goes at the
first multiple of 3 whose low tryte is free. A label names the address
of the next thing emitted: the centre of a word, or the tryte itself for
tryte data. Assembly starts so that the first word is centred on 0, the
reset address.

## Directives

| Directive | Effect |
|-----------|--------|
| `.org ADDR` | place the next word centred on `ADDR`, a multiple of 3 |
| `.align K` | place the next word centred on a multiple of 3^K |
| `.word E, ...` | one word per expression |
| `.tryte E, ...` | one tryte per expression |
| `.string "..."` | one code point per tryte, then a 0 tryte |
| `.ascii "..."` | the same without the terminator |
| `.zero N` | N zero trytes |
| `.words N` | N zero words, word aligned: a label on it names the first word's centre |
| `.equ NAME, E` / `.set` | define a constant |
| `.section NAME[, LOAD[, RUN]]` | switch sections (below) |
| `.include "PATH"` | assemble another file here (below) |
| `.assert E, "MESSAGE"` | stop with MESSAGE as the error if E is 0 |
| `.compress` / `.nocompress` | turn automatic bundling on and off from here (below); off at the start |
| `.gp E` / `.nogp` | in this section, from here: `gp` holds E, or nothing known (below) |

`.assert` is checked once every symbol is known, so it may use labels
defined after it. If assembly stops earlier, on a line that needs a
symbol nobody defined, an `.assert` before that line that already fails
is reported instead: it names the cause. A file meant only to be included can say what to
build instead:

```
        .assert defined(RAM_PAGES), "include/kernel.s is part of a build: run shell.s or demo.s"
```

Escapes, in strings and character literals: `\n`, `\t`, `\0`, `\\`,
`\'` and `\"`. Any other escape is an error, so a typo such as `\b`
does not quietly become `b`. Code points above 9,841
do not fit a tryte and are rejected until the text encoding (`isa.md`
12.2) is settled.

## Local labels

A label that is only digits can be defined any number of times. `1b`
refers to the nearest `1:` at or before the current line, so
`1: j 1b` jumps to itself, and `1f` to the nearest one after it:

```asm
puts:   li    t1, CONSOLE_OUT
1:      lt    t0, 0(a0)
        beqz  t0, 2f
        sw    t0, 0(t1)
        addi  a0, a0, 1
        j     1b
2:      ret
```

## Sections

Each section keeps its own place, so code and data can be written side
by side and land apart. Assembly starts in the section `text`, placed so
its first word is centred on 0. A new section needs an address the first
time it appears, and resumes where it left off after that:

```asm
        .section data, 2 * 19683   # placed once, here
greeting: .string "hello"
        .section text              # back to the code
        ...
        .section data              # and to the data
```

Like `.org`, a section address names where the next word is centred, so
tryte data placed there starts one tryte below it.

### Running somewhere other than where it loads

`.section NAME, LOAD, RUN` puts the section's trytes into the image at
`LOAD`, but lays it out at `RUN`: labels, branch offsets and `la` all use
the run address, the one the code executes at once paging maps it there.
A kernel loaded into physical memory and run at a virtual address gets
virtual labels this way, so `li label` and the debugger work in its own
terms:

```asm
        .section kernel, 1 * 19683, -3 * 19683   # load at page 1, run at page -3
kentry: ...
```

The two addresses must differ by a multiple of 3, so words are aligned in
both. `.org` and `.align` inside the section use run addresses. Overlaps
are checked where trytes load. The listing shows both addresses as
`RUN@LOAD` for such a section. A label written
before a `.section` line names the end of the section being left.
Sections do not move to make room for each other: give them addresses
that keep them apart, or the assembler reports the overlap.

## Includes

`.include "PATH"` assembles another file in place, as if its text were
pasted there. The path is relative to the file that contains the
directive. An included file can include others; including a file that
is already being included is an error. Errors name the file and line
they come from, as `lib/console.s:12: ...`.

Everything shares one symbol table, so an included file's labels and
constants are visible after the include, and a label used twice is
still an error.

## Images

`detrytus asm FILE -o OUT.dti` writes the assembled program as an image
file, with its labels and constants so the debugger can still name
addresses. `run` and `debug` take any mix of sources and images and load
them together. That lets a kernel and its programs be built separately:
they share no symbols, so they agree on addresses by convention (an
`.equ` in a shared include file is the natural place). Images that
overlap are refused. A name defined differently by two images, such as
two programs that each have a `start`, is left out of the merged symbol
table rather than resolved to the wrong one.

The format packs five trits into each byte; the layout is described at
the top of `rust/core/src/image.rs`.

## Operand forms

| Form | Used by |
|------|---------|
| `imm(reg)` | loads, stores, `jalr` |
| `(reg)` | atomics |
| `label` | branches and jumps: assembled as a word offset |
| table name or number | `tlut`, `amotlut`: `tmin`, `tmax`, `tmul`, `tsum`, `tcons`, `tany`, `tabs` |
| CSR name or number | `csrrw`, `csrrtmax`, `csrrtmin` |
| `pred, succ` | `fence`, as letters `r`, `w`, `i`, `o`; bare `fence` orders everything |

## Suffixes

| Suffix | On | Meaning |
|--------|----|---------|
| `.likely` / `.unlikely` | `b<cond>` | static prediction hint |
| `.n` / `.z` / `.p` | `br3` | predicted sign |
| `.aq` / `.rl` / `.aqrl` | atomics | memory ordering |

## Pseudo-instructions

| Pseudo | Expansion |
|--------|-----------|
| `nop` | `addi zero, zero, 0` |
| `mv rd, rs` | `addi rd, rs, 0` |
| `li rd, value` | `addi`, or `lui` + `addi` when the value needs more than 16 trits or is not known yet |
| `la rd, symbol` | `auipc` + `addi`; under `.gp`, `addi rd, gp, symbol − E` (below) |
| `lw rd, symbol`, `lt`, `sw rs2, symbol`, `st` | under `.gp` only: `lw rd, symbol − E(gp)` and so on |
| `neg rd, rs` / `not rd, rs` | `sub rd, zero, rs` |
| `sgn rd, rs` | `cmp rd, rs, zero` |
| `beq`, `bne`, `blt`, `bgt`, `ble`, `bge` | `b<cond>` |
| `beqz rs, L` etc. | `b<cond> rs, zero, L` |
| `j label` | `jal zero, label` |
| `call label` | `jal ra, label` |
| `ret` | `jalr zero, 0(ra)` |
| `csrr rd, csr` / `csrw csr, rs` | `csrrw` with `zero` |
| `tmin rd, a, b` etc. | `tlut` with the named table |
| `tabs rd, rs` | `tlut rd, rs, zero, tabs` |
| `tmap rd, rs, f` | `tlut` with the unary function `f`: a 3-trit value whose trits, least significant first, are the outputs for `-`, `0`, `+` |

`csrw csr, zero` (or `csrw csr, r0`) is an error. A CSR instruction
whose source is `r0` only reads the CSR (`isa.md` 7.12), so the
expansion would write nothing. To write zero, write from a register
that holds 0. The raw form `csrrw rd, csr, zero` stays legal, since it
is the read form that `csrr` expands to.

## The global pointer

`.gp E` tells the assembler that `gp` holds the value of E (`isa.md`
4.2) in the code that follows in the current section. Each section
keeps its own: switching away and back resumes it, a section starts
with none, and `.nogp` ends it. E may name labels defined later. Under
`.gp`:

- `la rd, symbol` is the single instruction `addi rd, gp, symbol − E`,
  which fits a slot (`c.lagp`) when the distance is a multiple of 3
  within ±363. `la gp, ...` stays an `auipc` pair, since that is how
  `gp` gets its value.
- `lw rd, symbol` (and `lt`, `sw rs2, symbol`, `st`) loads or stores
  the global itself: `lw rd, symbol − E(gp)`, with no register spent on
  its address. Outside `.gp` these forms are an error.
- A symbol more than 16 trits (±21,523,360) from E stops assembly.

The assembler takes E on trust: setting the register is the program's
job, before the first instruction that relies on it.

```asm
        .section kernel, 1 * 19683
        .gp   kgp
start:  la    gp, kgp              # auipc + addi: this is what sets gp
        la    t0, procs            # addi t0, gp, procs - kgp
        lw    t1, ticks            # lw t1, ticks - kgp(gp)
        addi  t1, t1, 1
        sw    t1, ticks
        ...
        .section kdata, 2 * 19683
ticks:  .word 0
kgp:                               # gp points here, next to the globals
procs:  .words 100
```

## Compressed instructions

The `c.` mnemonics of `isa.md` 6.1 write bundles by hand. Two in a row
share a word, the first in slot A; a lone one gets `c.nop` in slot B. A
label on the second, or any other statement between them (a directive
too), starts a new word, since a branch cannot enter slot B.

```asm
        c.li  a0, 5               # slot A of the word at 0
        c.addi a0, 1              # slot B
loop:   c.addi a1, -1             # a label: slot A of the next word
        c.bne a1, zero, loop      # slot B, branching back to loop
```

A `c.` instruction takes the operands of the full instruction it stands
for, less the ones its form fixes, so offsets, values and targets are
written as for the full instruction:

| Written | Is |
|---------|----|
| `c.mv rd, rs` | `addi rd, rs, 0` |
| `c.add rd, rs` (and `sub`, `cmp`, `mul`, `min`, `max`, `tmin`, `tmax`) | `add rd, rd, rs` |
| `c.lw rd, (rs)`, `c.lt rd, (rs)` | `lw rd, 0(rs)` |
| `c.sw rs2, (rs)`, `c.st rs2, (rs)` | `sw rs2, 0(rs)` |
| `c.jr rs`, `c.jalr rs` | `jalr zero, 0(rs)`, `jalr ra, 0(rs)` |
| `c.sys`, `c.tret`, `c.nop` | `sys`, `tret`, `addi zero, zero, 0` |
| `c.addi rd, imm` (and `shi`, `cmpi`) | `addi rd, rd, imm` |
| `c.li rd, imm`, `c.lip rd, value` | `addi rd, zero, imm` |
| `c.lwsp rd, off`, `c.swsp rs2, off` | `lw rd, off(sp)`, `sw rs2, off(sp)`: `off` in trytes |
| `c.lagp rd, off` | `addi rd, gp, off`: `off` in trytes |
| `c.beqz rs, label`, `c.bnez rs, label` | `beq rs, zero, label` |
| `c.beq rs1, rs2, label` (and `bne`, `blt`, `ble`) | `beq rs1, rs2, label` |
| `c.j label`, `c.jal label` | `jal zero, label`, `jal ra, label` |

The full instruction must fit the form, or assembly stops:
`addi a0, a0, 200 does not fit c.addi`. A branch in either slot counts
from the bundle's address. Listings and the debugger show a bundle as
`c.li a0, 5 | c.addi a0, 1`; a trace shows each slot as the full
instruction it ran.

### Automatic bundling

Between `.compress` and `.nocompress` (or the end of the program), the
assembler puts ordinary instructions into bundles itself, by the same
rules as `c.` instructions: two neighbours that both fit a form share a
word, and a label or any other statement between them keeps them apart.
An instruction that fits but has no partner stays a full word, since a
bundle with `c.nop` saves nothing. Instructions of more than one word
(`la`, `li` of a large or forward value) are never paired.

Whether a branch fits can depend on how much the code before it shrank,
so layout repeats until every pair fits, each time leaving out the
instructions that did not. A branch that fits nowhere stays full.

Bundling moves labels, so leave out code whose layout matters with
`.nocompress`: a vectored trap table, one entry per word, or code whose
size an `.assert` checks. A `sys` handler must resume at `tval`, not
`tpc + 3` (`isa.md` 8.4), before code that calls it is bundled.

## CSR names

Ring +1's trap bank uses an `m` prefix and ring 0's an `s` prefix:
`mtstatus`, `mtvec`, `mtpc`, `mtcause`, `mtval`, `mscratch`, `mie`, `mip`,
`mtimecmp`, and `stvec` and so on. The other CSRs use their names from
`isa.md` 8.2 and 8.6.
