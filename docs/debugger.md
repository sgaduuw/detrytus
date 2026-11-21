# detrytus debugger

`detrytus debug FILE` assembles a program and stops before its first
instruction. Every address it prints carries the nearest label, so
`57 <bsearch+3>` is three trytes into `bsearch`.

```bash
detrytus debug examples/bsearch.s
```

```bash
detrytus debug examples/bsearch.s -b bsearch -x commands.txt --batch
```

`-b ADDR` sets a breakpoint before starting (repeatable), `-x FILE` runs
debugger commands from a file first, and `--batch` exits once they are
done, which makes sessions scriptable. `--input FILE` queues the file's
text on the console's input, as for `run`, and again on every `restart`.
Unlike `run`, the debugger keeps the console open once that text is
read, so `input` can add more; `eof` closes it.
`--harts N`, `--plug TIME:HART`
and `--unplug TIME:HART` set up harts as for `run` (see Several harts,
below).

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
| `hart [N]` | | list the harts, or look at hart N |
| `plug HART [TIME]` | | plug a hart in at TIME (default: the next step) |
| `unplug HART [TIME]` | | ask for a hart to go at TIME (default: the next step) |
| `input TEXT` | | queue TEXT on the console's input; `\n` newline, `\t` tab, `\s` space, `\\` backslash |
| `eof` | | close the console's input: once what is queued is read, console in reads `CONSOLE_CLOSED` (−2), not `CONSOLE_EMPTY` (−1); `restart` opens it again |
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

## Several harts

A step is a round of the machine (`docs/platform.md`, Time): every
running hart runs one instruction. `step 3` is three rounds, whichever
hart is in focus.

One hart is **in focus**: `regs`, `csr`, `print` (`$pc`, `$a0`),
`examine`, `disas`, `watch` values and `next` use it, and addresses are
translated with its page table. `hart` lists every present hart, marking
the one in focus with `*`, with its pc and ring once it has run; `hart
N` moves the focus to hart N.

A breakpoint stops any hart about to run the instruction at its address
(the hart in focus is checked first); a watchpoint stops on any hart's
access; `catch` on any hart's trap. Each moves the focus to the hart
that stopped, and when there can be several harts, messages and the
`=>` line name it: `stopped: breakpoint 1 at 126 <worker> on hart 1`.
`next` waits for the call to return on the hart in focus.

`plug` and `unplug` schedule hot-plug at a time on the shared clock, as
`--plug` and `--unplug` do. `info` lists the changes, and `restart`
makes them again, as it keeps breakpoints. Focus returns to hart 0 on
`restart`, and when the hart in focus is removed.

## Looking never changes the machine

Inspection has no side effects. `examine`, `disas` and watchpoint values
read memory without filling the TLB, and they never read device
registers, whose reads can consume input: those show as `<unmapped>`.
`csr` reads with ring +1's view without changing the stopped program's
ring.
