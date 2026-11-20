# detrytus debugger

`detrytus debug FILE` assembles a program and stops before its first
instruction. Every address it prints carries the nearest label, so
`57 <bsearch+3>` is three trytes into `bsearch`.

```bash
uv run detrytus debug examples/bsearch.s
```

```bash
uv run detrytus debug examples/bsearch.s -b bsearch -x commands.txt --batch
```

`-b ADDR` sets a breakpoint before starting (repeatable), `-x FILE` runs
debugger commands from a file first, and `--batch` exits once they are
done, which makes sessions scriptable.

The Rust `detrytus` command (`rust/cli`) has the same debugger, with the
same commands and output, line for line:

```bash
cargo run --release -p detrytus-cli -- debug examples/bsearch.s
```

## Commands

| Command | Alias | Does |
|---------|-------|------|
| `step [N]` | `s` | run N instructions (default 1) |
| `next` | `n` | step, running a whole call as one step |
| `continue` | `c` | run to a breakpoint, watchpoint, caught trap or power-off |
| `restart` | | reload the program on a fresh machine; stopping points stay |
| `break ADDR` | `b` | stop before the instruction at ADDR |
| `watch ADDR [r\|w\|rw]` | `w` | stop after an access to ADDR (default: writes) |
| `catch [all\|off\|CAUSE ...]` | | stop when a trap is taken |
| `delete [ID]` | `d` | remove one stopping point, or all |
| `info` | `i` | list breakpoints, watchpoints and caught traps |
| `regs` | | every register, the pc and the ring |
| `csr NAME` | | one CSR, such as `mtcause` or `ptbr` |
| `print EXPR` | `p` | evaluate an expression, in decimal and balanced ternary |
| `examine ADDR [N]` | `x` | N words from ADDR (default 4) |
| `xp ADDR [N]` | | N words of physical memory from ADDR, untranslated |
| `disas [ADDR] [N]` | | disassemble N words (default: 8 from the pc) |
| `trace on\|off` | | print every instruction as it runs |
| `quit` | `q` | leave |

`help CMD` explains any of them.

## Addresses and expressions

ADDR and EXPR are assembler expressions (`docs/assembler.md`): labels,
`.equ` constants, numbers in any of the assembler's forms, and `+ - * /`.
`$name` reads a register by any of its names (`$a0`, `$sp`, `$r-3`) and
`$pc` the program counter, so `x $sp 6` shows the top of the stack.

Addresses are virtual, as the stopped program sees them: in a ring
running with paging, `examine` and `watch` go through its page table.
`xp` is the exception: it reads physical memory as it is, which is what
a kernel's own data structures need once the machine has stopped.

`watch` on an aligned address (a multiple of 3) watches the word
centred there; on any other address, the single tryte.

`catch` takes trap cause numbers (`4`, `-1`) or names: `illegal`,
`misaligned`, `access`, `syscall`, `breakpoint`, `fetch-fault`,
`load-fault`, `store-fault`, `timer`. It stops once the trap has been
taken, at the first instruction of the handler.

## Looking never changes the machine

Inspection has no side effects. `examine`, `disas` and watchpoint values
read memory without filling the TLB, and they never read device
registers, whose reads can consume input: those show as `<unmapped>`.
`csr` reads with ring +1's view without changing the stopped program's
ring.
