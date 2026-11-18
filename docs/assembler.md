# detrytus assembler

`detrytus asm FILE` prints a listing; `detrytus run FILE` assembles and
runs. The instruction set is in [`isa.md`](isa.md); this page covers the
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
toward zero.

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
| `.equ NAME, E` / `.set` | define a constant |

String escapes: `\n`, `\t`, `\0`, `\\`, `\"`. Code points above 9,841
do not fit a tryte and are rejected until the text encoding (`isa.md`
12.2) is settled.

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
| `la rd, symbol` | `auipc` + `addi` |
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

## CSR names

Ring +1's trap bank uses an `m` prefix and ring 0's an `s` prefix:
`mtstatus`, `mtvec`, `mtpc`, `mtcause`, `mtval`, `mscratch`, `mie`, `mip`,
`mtimecmp`, and `stvec` and so on. The other CSRs use their names from
`isa.md` 8.2 and 8.6.
