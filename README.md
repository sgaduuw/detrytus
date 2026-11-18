# detrytus

A balanced-ternary CPU architecture, picking up where the Soviet Setun
(1958) and Setun-70 (1970) left off and adding the features a modern
core has: a RISC load/store design, a zero register, three privilege
rings, traps, CSRs and atomics.

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

## License

MIT, see [`LICENSE`](LICENSE).
