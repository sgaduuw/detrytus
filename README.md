# detrytus

A balanced-ternary CPU architecture, picking up where the Soviet Setun
(1958) and Setun-70 (1970) left off and adding what a modern operating
system expects from a core: a RISC load/store design, position-independent
code, three privilege rings with trap delegation, paged virtual memory,
a timer, acquire/release atomics, floating point and cache control.

The target is an operating system of its own (a microkernel, as a
separate project). Running Linux is not a goal: C and the kernel assume
binary integers and 8-bit bytes throughout.

The name is detritus with a tryte in it.

## Status

Design stage. The instruction set is specified as a draft in
[`docs/isa.md`](docs/isa.md). Next up is a Python emulator and
assembler written against that spec; a Rust implementation may follow
once the design settles.

## The shape of it

- Trits take the values −1, 0, +1. A tryte is 9 trits, a word is 27.
- 27 registers indexed −13..+13, with `r0` hardwired to zero.
- Fixed 27-trit instructions.
- A compare yields a trit, and `br3` branches three ways on it.
- One integer type: no sign bit, no unsigned variants, and `not` and
  `neg` are the same instruction.
- One `tlut` instruction expresses any of the 19,683 two-input trit
  functions.
- Pages, words and tables are centred on their addresses, so an address
  splits into page number and offset just by cutting trits.
- The sign of a physical page number picks RAM, boot firmware or
  devices; the sign of a trap cause picks interrupt or exception.

## License

MIT, see [`LICENSE`](LICENSE).
