# Objects and the linker

A program can be built from separately assembled files. `detrytus asm
-c` assembles one source into an **object**, whose addresses are not
all settled yet, and `detrytus link` puts objects together into an
ordinary image. A source assembled whole, as [`assembler.md`](assembler.md)
describes, needs neither.

```sh
detrytus asm -c main.s -o main.dto
detrytus asm -c lib.s -o lib.dto
detrytus link main.dto lib.dto -o prog.dti     # an image, as asm -o writes
detrytus run prog.dti
```

`link` takes sources as well as objects, assembling each as an object,
so `detrytus link main.s lib.s -o prog.dti` does the same in one step.
It compiles Trice files (`.tc`) each on its own as well, as `detrytus tc
-c` does ([`trice.md`](trice.md), section 13).
`run` and `debug` link the objects among their files, then load the
result with the rest as usual. Without `-o`, or with `--map`, `link`
prints where each object's part of each section went, named by its
first label, the parts it left out as unused, and the address of every
global symbol.

## Writing a source for an object

An object is assembled as a whole source is, with three differences.

**Sections without an address are the linker's.** `.section NAME` the
first time, with no address, starts a section the linker places, as is
the initial section `text`. Inside one, addresses count from the
section's base, wherever the linker puts it: `.align` asks for a base
that is a multiple of a power of 3, and `.org` is an error. A section
given an address, `.section NAME, LOAD[, RUN]`, is placed where it says,
as in a whole source.

**Names the source does not define are external.** The linker finds
them among the other objects' global symbols. A name that no object
makes global is an error then, reported at the line that used it.

**`.global NAME, ...` makes names visible to other objects**: labels,
and constants defined with `.equ`. Everything else stays local to its
object. A source assembled whole accepts `.global` and ignores it, so
the same file builds either way.

**`.part` starts a piece the linker may leave out.** The code and data
after it, up to the next `.part` or the end, are a part of the current
section of their own, which the linker keeps only if something it keeps
refers to it. Everything before a section's first `.part` is always
kept, as is every section given an address, so a program's start code
and whatever it reaches stay, and a library function nothing calls
goes. Switching to another section and back resumes the latest part.
A part counts as used when a kept part branches to it, loads its
address or holds it in a `.word`, by a label of its own or a global
name it defines. A source assembled whole accepts `.part` and ignores
it.

```asm
        .part
        .global putc
putc:   ...                     # kept if some kept code calls putc
        ret
        .part
        .global puts
puts:   ...                     # left out otherwise
```

Trice puts each function and each global of a library in a part of its
own, and each assembly file it imports ([`trice.md`](trice.md), section
13), so a program linked with a library gets only the library code it
calls.

### What the linker can finish

A value that depends on where the linker puts things must be one
address plus a constant, or one address minus another plus a constant,
such as a symbol's distance from `gp`'s value (assembler.md, The global
pointer). It may go in a `.word`, in the immediate of an instruction
(`addi`, a load or store, `li` and `la`), or be a branch or jump target.
It cannot go in a `.tryte`, a CSR number or a count, be multiplied by
another such value, divided or compared, or be checked by an `.assert`.
Two addresses in the same section subtract to a constant, so `end -
start` within a section works anywhere a number does.

`li` of such a value always takes the long form (`lui` + `addi`), and
an instruction the linker finishes is never put in a compressed bundle,
neither by `.compress` nor by a `c.` mnemonic: whether it would fit a
slot is not known until it is linked.

## How the linker places sections

Every object's sections of the same name become one section of the
program, in the order the objects are given:

- If an object gave the section an address, its part goes there, and
  the other objects' parts follow it. At most one object may place a
  section.
- A section no object places follows the section before it, so that
  sections come in the order they first appear. The first one starts
  with its first word centred on 0, where an assembled source starts.
- Each part starts at the first base its alignment allows, words
  centred as always (assembler.md, Layout).

There is no linker script and no address on the command line: as in a
whole source (CONTEXT.md, Sections are placed explicitly), an address
is given once, in the source that owns it. trylobite's `lib/start.s`,
which places a program's section at `V_USER`, is that source for every
program linked with it.

Parts started by `.part` that nothing kept refers to are left out
first (Writing a source for an object); the rest are placed as above,
each `.part` as one more part of its section, in the order they come
in their object.

Parts that land on each other are an error, as are a global defined in
two objects and a value that does not fit the field it goes in.

The image keeps every global symbol, and the local ones whose names no
other object uses for another value, so the debugger names addresses in
linked programs too.

## The object format

An object is text, one fact to a line, so it can be read and compared
without a tool:

```text
detrytus object 1
section text 1 17
data text -1 1978 0 -9 7168 1955 265 7168 9065 84 6551 -1 0 -11 -4 0 5467 4358 2757
section data 1 2
data data -1 42 0 0
reloc text 6 branch.imm 0 +puts @ 3 main.s
symbol global label puts 0 in text
symbol local constant POWER_OFF -39366
```

| Line | Says |
|------|------|
| `detrytus object 1` | the format and its version, always first |
| `section NAME ALIGN END [at SHIFT]` | a section: its base a multiple of 3^ALIGN; END the tryte after its last. With `at`, its source placed it, its addresses are where it runs and SHIFT is its load address minus its run address. `NAME:K`, such as `text:2`, is a part `.part` started, which goes into section NAME |
| `data NAME ADDR TRYTE...` | trytes from ADDR up |
| `reloc NAME ADDR KIND ADDEND [+BASE] [-BASE] @ LINE [FILE]` | what the linker writes at ADDR |
| `symbol global\|local label\|constant NAME VALUE [in SECTION]` | a name; `in` a section the linker places, VALUE counts from its base |

Addresses in a section the linker places count from its base, so its
first tryte is at −1. A relocation's value is ADDEND, plus the first
BASE, minus the second; a BASE is `[SECTION]`, the base of one of the
object's own sections, or the name of a global symbol in any object.
LINE and FILE name the source line for messages. KIND says where the
value goes:

| Kind | Written as | Used by |
|------|------------|---------|
| `word` | the word at ADDR, as `.word` stores it | `.word` |
| `imm` | the `imm` field of the instruction at ADDR | `addi`, loads, stores |
| `branch.FIELD` | the value less ADDR, in words, in FIELD (`off`, `offn`, `offp` or `imm`) | branches, `br3`, `jal` |
| `pair` | split over the `lui` at ADDR and the `addi` after it | `li` |
| `pcpair` | the value less ADDR, split over the `auipc` at ADDR and the `addi` after it | `la` |

A pair splits its value as `li` and `la` do: the low 8 trits, balanced,
in the `addi`, and the rest in the first instruction, so neither part
needs a carry.
