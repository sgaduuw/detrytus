# Emulator platform

What the emulator puts around the CPU. The ISA leaves these choices to
the platform (`isa.md` 8.5 and 12.3).

## Memory map

| Physical page | Contents |
|---------------|----------|
| 0 | boot page: the program is loaded here and runs from address 0, or, with a disk and no program, the boot ROM (Booting from a disk, below); read and execute only |
| 1 .. 59049 | RAM (`--ram-pages` changes the count, up to 193,710,244; the RAM size register reads it) |
| -1 | console |
| -2 | power control |
| -3 | hart controller: one slot per hart (Harts, below) |
| -4 | hart controller: this hart's slot, group send, events |
| -5 .. -12 | reserved for more harts |
| -13 | interrupt controller (Device interrupts, below) |
| -14 | block device (The block device, below) |
| -15 | binary floating point coprocessor (below) |
| -16 | network device (The network device, below) |

A page is 19,683 trytes centred on a multiple of 19,683, so page `p`
spans `p·19683 − 9841` .. `p·19683 + 9841`.

RAM can be any number of pages from 1 to 193,710,244, (3¹⁸ − 1) / 2:
every positive page number, up to the top of the address space. The
emulator allocates a page's host memory the first time something writes
to it, so a large machine costs only what its programs use, and
`detrytus run` and `detrytus debug` give a machine 59,049 pages (3¹⁰,
about 1.16 billion trytes) unless `--ram-pages` says otherwise. Software
learns the size from the RAM size register rather than being built for
one machine. The loader may place data in
RAM pages too; only the program writes are checked.

## Devices

| Address | Register | Behaviour |
|---------|----------|-----------|
| −19683 | console out | writing a Unicode code point prints it |
| −19680 | console in | reading returns the next input code point, or a negative state when there is none (Console input states, below; test the sign: a code point is never negative) |
| −19677 | console rows | read-only: the rows of the terminal the console is shown on, or 0 if not known (The console's size, below) |
| −19674 | console columns | read-only: its columns, or 0 if not known |
| −39366 | power off | writing any value stops the machine; the value becomes the exit code |
| −39363 | RAM size | read-only: how many RAM pages the machine has (pages 1 to N are RAM); writes are ignored |
| −39360 | clock | read-only: the real-time clock, whole seconds since 1970-01-01 00:00:00 UTC; reading it latches clock micros (The real-time clock, below) |
| −39357 | clock micros | read-only: the microseconds, 0 to 999,999, past the second this hart's last read of clock gave |
| −39354 | monotonic | read-only: the monotonic clock, whole seconds since the machine started; reading it latches monotonic nanos (The monotonic clock, below) |
| −39351 | monotonic nanos | read-only: the nanoseconds, 0 to 999,999,999, past the second this hart's last read of monotonic gave |

Console input is the text `--input FILE` names, queued before the
first step, plus whatever the debugger's `input` command adds, or, with
a live console (`run` at a terminal, or `--live`), what is typed, as it
is typed. Once it is used up, reads say whether more may come (Console
input states, below). With `--input` a run's input is fixed in advance,
so runs stay deterministic and nothing waits for a person to type.

### The console's size

The console has a size, as a virtual console does: two read-only
registers say how many rows and columns of characters the terminal
showing it has, so a program can lay out a screen to fit. Both read 0
when the size is not known, and software then picks its own (24 by 80
is the usual guess). `run --console-size ROWSxCOLUMNS` sets them; a
live console on a terminal without that flag follows the terminal,
asking `stty size` at the start and every half second, so a window
made larger or smaller shows within a second. Nothing interrupts on a
change: a program reads the registers when it draws. Without a
terminal and without the flag (`--input`, a pipe, the debugger) they
read 0, so runs stay deterministic.

## Time

`time` advances by one per step, so runs are deterministic. `wfi` skips
`time` forward to the earliest enabled timer compare, since a lone hart
has nothing else to wait for.

With several harts, each step is a round: `time` advances by one, then
every running hart takes one step, in hart-number order, so every hart
reads the same clock and runs stay deterministic. A hart started during
a round first runs in the next one. `--max-steps` and `--stats` count
rounds. With one hart this is the same machine as before.

`wfi` waits: the hart sits rounds out until an interrupt is pending and
enabled in its `ie` (in either bank, whatever the rings and `IE` bits).
`wrs` waits the same way, and also ends once the hart's reservation is
gone (another hart stored to the word) or `time` reaches its deadline;
with no reservation it does not wait at all. When every running hart is
waiting, time skips forward to the earliest moment one could wake: an
enabled timer compare, a `wrs` deadline or a scheduled hot-plug change. If nothing could ever wake them, they all return from `wfi` at
once, which the ISA allows.

With a live console (`run` at a terminal, or `--live`) that skip takes
real time: the clock moves over idle time at about a million ticks a
second, 20,000 ticks every 20 ms, waiting for a key in between. A
program that sleeps a million ticks then sleeps about a second, as a
person watching expects, rather than not at all. Running, the clock
still moves one tick per step, however fast that is; a run with
`--input` skips at once, as before, and stays deterministic.

## The real-time clock

`time` counts steps, which says nothing about the time of day. The
real-time clock does: two read-only registers in the power page give
Unix time, seconds since 1970-01-01 00:00:00 UTC, and the microseconds
past that second. There are no time zones and no leap seconds in it, as
in Unix time; turning seconds into a date is software's job.

- **Reading.** Load clock, then clock micros. Reading clock latches the
  microseconds of the same moment for the hart that read it, so the
  pair is consistent however a second turns over between the loads, and
  harts reading at once do not disturb each other's latch. Clock micros
  before a hart's first read of clock is 0. The seconds are rounded
  down, so the microseconds are never negative, before 1970 too.
- **Where the time comes from.** By default the host's clock, read at
  each access. `run --clock SECONDS` (and `debug --clock SECONDS`)
  instead starts the clock SECONDS after the epoch at tick 0 and moves
  it a microsecond a tick, the rate a live console paces idle time at,
  so a run that reads it is still the same every time.
- **Writes are ignored.** Setting the time of day is the operating
  system's business: it keeps its own offset from the clock.

`examples/clock.s` reads and prints it.

## The monotonic clock

The real-time clock follows the host's, which a person or NTP can set
back or forward, and an operating system adds its own offset to it. To
measure how long something took, software needs a clock that only moves
forward at a steady rate: the monotonic clock, two read-only registers
after the real-time clock's, whole seconds since the machine started
and the nanoseconds past that second.

- **Reading.** As with the real-time clock: load monotonic, then
  monotonic nanos. Reading monotonic latches the nanoseconds of the
  same moment for the hart that read it; monotonic nanos before a
  hart's first read of monotonic is 0.
- **Where the time comes from.** By default the host's monotonic clock
  (Rust's `Instant`), from when the machine was made, so setting the
  host's time of day does not move it. With `--clock SECONDS` it is
  `time` instead, a microsecond (1,000 nanoseconds) a tick, so runs
  stay deterministic.
- **Writes are ignored.**

Unlike `time`, which counts steps and runs as fast as the emulator
does, the monotonic clock is real time.

## Harts

isa.md 5.6 describes harts; this is the reference platform's side.

The platform allows up to 729 harts, numbered −364..+364, one 27-tryte
slot each in page −3. `run --harts N` (default 1) makes N harts present
at reset, numbered outward from 0: 0, −1, +1, −2, +2, and so on, so 3
harts are a triad, 9 the middle nine, 27 the middle 27. Pages −5..−12
are reserved so a later platform can grow past 729.

`examples/harts.s` starts the two other harts of a triad and greets them
by message.

### Per-hart slots (page −3)

Hart `h`'s slot is centred on `S = −59049 + 27·h`. Every register is one
word, centred on the address given.

| Address | Register | Access | Behaviour |
|---------|----------|--------|-----------|
| S−12 | STATE | ro | −1 absent, 0 stopped, +1 running |
| S−9 | START_PC | rw | where the hart starts |
| S−6 | START_ARG | rw | put in `a0` when it starts |
| S−3 | CONTROL | wo | +1 starts a stopped hart; −1 stops the hart writing it (own slot only) |
| S | SEND | wo | queues the word, which must be 0 or more, in hart `h`'s mailbox |
| S+3 | RECEIVE | ro | own slot only: takes the oldest message, or reads `MAILBOX_EMPTY` (−1) if there is none |
| S+6 | COUNT | ro | messages waiting, 0..9 |
| S+9, S+12 | — | — | reserved, read 0 |

- **Starting.** Writing +1 to CONTROL of a stopped hart starts it in the
  reset state of isa.md 5.4: ring +1, `pc` = START_PC, `a0` = START_ARG,
  other registers 0, CSRs at reset values, an empty TLB and mailbox, no
  reservation. Writing +1 to a running or absent hart does nothing.
- **Stopping.** A hart writing −1 to its own CONTROL stops at once; its
  registers, TLB, reservation and messages are discarded. Writing −1 to
  another hart's CONTROL does nothing: removal is cooperative.
- **Mailbox.** Nine words deep. A SEND is accepted only if hart `h` is
  running, its mailbox has room and the word is 0 or more; otherwise it
  is rejected and the word is dropped. Messages are never negative, so
  RECEIVE's negative `MAILBOX_EMPTY` cannot be mistaken for one: a
  receiver tests the sign (`bltz`), never the number. The result of the hart's last send, to any slot or
  group, is in LAST (below), so a sender checks it when delivery
  matters; a send never fails silently.
- **Doorbell.** Hart `h`'s software interrupt (cause −3, `ip` trit 3)
  is pending while COUNT > 0, ORed with the bit software can set
  itself. Reading RECEIVE until it is empty clears it. It appears in
  ring +1's `ip`, or ring 0's when `ideleg` trit 3 is `+`.
- **Own slot only.** RECEIVE and CONTROL −1 act only from the hart that
  owns the slot; from another hart a read of RECEIVE gives
  `MAILBOX_EMPTY` and takes nothing.
- **Ordering.** The slots are MMIO: a sender fills any shared buffer,
  then `fence w, o`, then SEND; a receiver reads RECEIVE, then
  `fence i, r`, then reads the buffer.

### This hart, group send, events (page −4)

Page −4 is centred on `C = −78732`. Its middle 27 trytes, `C−12 .. C+12`,
are a window onto the slot of whichever hart accesses them, so code can
reach its own mailbox without computing its address from `hartid`.

| Address | Register | Access | Behaviour |
|---------|----------|--------|-----------|
| C−12 .. C+12 | own slot | as above | the accessing hart's slot in page −3 |
| C+27 | TARGET | rw, per hart | a hart number naming a subtree |
| C+30 | SPAN | rw, per hart | `k`, 0..6 (a write outside is clamped): the subtree is every hart whose number agrees with TARGET except in its lowest `k` trits |
| C+33 | GSEND | wo | sends the word, 0 or more, to every running hart in the subtree except the sender; a negative word reaches none and is a refusal |
| C+36 | LAST | ro, per hart | the last send's result: +1 accepted by all, −1 refused by at least one, 0 none yet |
| C−27 | HARTS | ro | slots on this platform: 729 |
| C−30 | EVENT_HART | rw | the hart that receives hot-plug events; 0 at reset |
| C−33 | EVENT | ro | takes the oldest hot-plug event: `kind + 3·h`, 0 if none |

- **Groups.** With SPAN `k`, TARGET `t` names the `3ᵏ` slots whose
  numbers agree with `t` above trit `k`: SPAN 0 is hart `t` alone, SPAN 1
  its triad, SPAN 2 its nine, SPAN 6 every hart. A group send skips
  absent and stopped harts without counting them as refusals; a full
  mailbox among the running ones is a refusal, and that hart gets
  nothing while the others still receive the word.
- **Hot-plug events.** Kind +1: hart `h` has been added and is stopped,
  ready to start. Kind −1: removal of hart `h` is requested; the kernel
  moves its work away and the hart stops itself, and the platform
  removes it as it stops. A hart already stopped when removal is asked
  for goes at once (the event still says so). Hart 0 cannot be removed.
  The external interrupt (cause −2) of hart EVENT_HART is pending while
  events are queued, in ring +1's `ip` or ring 0's as `ideleg` trit 2
  says.
- **Scheduling hot-plug.** `run --plug TIME:HART` adds a hart and
  `--unplug TIME:HART` asks for one to go, when `time` reaches TIME, so
  runs stay deterministic. Each may be given more than once.

## Device interrupts and a block device

Two devices that isa.md 12.3 lists. With them a kernel can sleep until
console input arrives instead of polling, and has storage for a file
system. Neither changes the ISA: devices raise the existing external
interrupt, cause −2. CONTEXT.md gives the reasoning and what was
rejected.

### Device interrupts (page −13)

A small interrupt controller gathers device interrupts and raises
cause −2 on the hart each is routed to. It is a pared-down RISC-V PLIC:
no priorities, one target hart per source, claim and complete.

**Sources** are numbered from +1; source 0 means none, which is safe
because no device is source 0.

| Source | Device | Pending while |
|--------|--------|---------------|
| 1 | console input | a read of console in would not return `CONSOLE_EMPTY` |
| 2 | block device | a finished command has not been acknowledged |
| 3 | network device | a finished send or receive has not been acknowledged |

Page −13 is centred on `I = −255879`. Source `s` has a three-word group
centred on `I + 9·s`; group 0, at `I`, holds the per-hart registers.
Every register is one word, centred on the address given.

| Address | Register | Access | Behaviour |
|---------|----------|--------|-----------|
| I−3 | CLAIM | ro, per hart | takes the lowest-numbered source that is pending, enabled, routed to the reading hart and not in service; marks it in service and returns its number, or 0 if there is none |
| I | COMPLETE | wo, per hart | ends service of the source written; if it is still pending it can be claimed again. Writing a source not in service does nothing |
| I+3 | SOURCES | ro | the highest source number on this platform: 3 |
| I+9·s−3 | PENDING(s) | ro | +1 while source `s` wants attention, else 0 |
| I+9·s | ENABLE(s) | rw | +1 lets source `s` interrupt, 0 masks it; 0 at reset. Other values are stored as 0 |
| I+9·s+3 | ROUTE(s) | rw | the hart source `s` interrupts; 0 at reset |

Groups of sources this platform does not have read 0 and ignore
writes.

- **Raising the interrupt.** Hart `h`'s cause −2 is pending while some
  source is pending, enabled, routed to `h` and not in service, or, as
  now, while `h` is EVENT_HART and hot-plug events are queued. It
  appears in ring +1's `ip`, or ring 0's when `ideleg` trit 2 is `+`.
- **Handling.** A handler reads CLAIM until it reads 0, serves each
  source it gets, and writes each to COMPLETE. A source in service does
  not interrupt again until it is completed, so a handler is not
  re-entered for the source it is serving.
- **Level-triggered.** A source stays pending until the device's
  condition clears, so nothing is lost between a claim and a complete.
  A source that is still pending at COMPLETE can be claimed again at
  once.
- **Routing to an absent or stopped hart** is allowed; the source
  simply interrupts nobody until it is routed elsewhere or the hart
  runs.
- **Hot-plug stays outside the controller** for now, on the same cause,
  so existing hot-plug handlers keep working. A handler for cause −2
  checks EVENT as well as CLAIM.

### Console input states

Console in reads one of three things. The two "no character" states
are named; code compares with the names, never with the numbers.

| Console in reads | Name | Meaning |
|------------------|------|---------|
| 0 or more | | the next code point, taken from the input |
| −1 | `CONSOLE_EMPTY` | nothing yet: more may come |
| −2 | `CONSOLE_CLOSED` | closed: nothing more will come |

- **Test the sign** for "is there a character": any negative read is
  none. Only code that sleeps until input arrives needs to tell the two
  apart.
- **`run` closes the console** once its `--input` is used up, or from
  the start without `--input`, since nothing can type into a run;
  except with a live console, which closes at Ctrl-D (on a terminal) or
  when the pipe ends.
- **`debug` keeps it open**, so `input` can add more; the debugger's
  `eof` command closes it, and `restart` opens it again.
- **Source 1** is pending while input is waiting or the console is
  closed. Closed is a lasting condition, so a kernel masks source 1 once
  it has seen `CONSOLE_CLOSED`, and whenever nobody is reading.

### The block device (page −14)

A disk that moves whole blocks between itself and RAM on its own, then
raises source 2. **A block is one page**, 19,683 trytes, so each block
fills or empties exactly one physical page. One command moves a run of
consecutive blocks, so a large transfer is one command and one
interrupt.

Page −14 is centred on `D = −275562`.

| Address | Register | Access | Behaviour |
|---------|----------|--------|-----------|
| D−9 | BLOCKS | ro | how many blocks the disk has; 0 with no disk |
| D−6 | BLOCK | rw | the first block to move |
| D−3 | COUNT | rw | how many consecutive blocks to move; 1 at reset |
| D | LIST | rw | the physical address of COUNT words: block BLOCK + i moves to or from the page whose number is word i |
| D+3 | COMMAND | wo | `BLOCK_READ` (+1) copies the blocks into the pages, `BLOCK_WRITE` (−1) copies the pages to the blocks; ignored while busy, and other values are ignored |
| D+6 | STATUS | ro | `BLOCK_BUSY` (+1), `BLOCK_DONE` (0) or `BLOCK_FAILED` (−1) |
| D+9 | ACK | wo | any write clears the finished state, which ends source 2's interrupt |

- **The sign of COMMAND is the direction**, as the sign of a cause is
  its kind.
- **Scatter list.** LIST is read without translation when the command
  starts, so the pages need not be next to each other. A kernel builds
  the list in RAM and points LIST at it.
- **Timing.** A command finishes 1,000 + 100·COUNT rounds after it is
  given, and copies everything at that moment. Eight blocks in one
  command take 1,800 rounds instead of 8,000, much as a seek dominates
  on a real disk, so gathering a large transfer into one command pays,
  while a small one is still a single block. Runs stay deterministic,
  and a kernel has a real wait to fill with other work. A program that
  touches the pages while the command is busy sees them change, or has
  its changes written, only when it finishes.
- **Failure.** A command fails, and STATUS reads `BLOCK_FAILED`, when
  COUNT is below 1, a block in the run is not on the disk, or a list
  word is not a RAM page (the boot page and device pages are not). A
  failed command moves nothing, takes no time, and is finished at once,
  so it still raises source 2.
- **Finished state.** A command that finishes, by success or failure,
  sets the finished state; source 2 is pending until ACK. STATUS keeps
  the outcome until the next command.

**The backing file.** `run --disk FILE` and `debug --disk FILE` attach
a disk. Each tryte is two bytes, a little-endian signed 16-bit number
from −9,841 to 9,841, so block `n` starts at byte `n × 39,366` and the
file can be read and patched with ordinary tools. BLOCKS is the file's
length in blocks, rounded up; `--disk-blocks N` sets it instead,
creating the file if it is missing. Past the file's end, blocks read as
zeros, and writes extend it. A run refuses a file holding a value that
is not a tryte. A command's blocks reach the file in one host write
when it finishes, so a crash keeps whole commands, never half of one.
The debugger's `restart` does not undo writes already made to the file.

## Booting from a disk

`run --disk FILE` and `debug --disk FILE` with no program put the boot
ROM in the boot page instead: a small program, `rust/cli/src/boot.s`,
that loads an image from the disk and starts it, much as a PC's
firmware reads a boot sector. A disk does not say what is on it, so a
**boot record** in block 0 does.

**The boot record** is words 3 to 8 of block 0, the trytes at offsets
−9,832 .. −9,815 from the block's centre. Words 0 to 2 and everything
after word 8 are left to whatever else uses block 0, such as a
filesystem's header.

| Offset | Word | Name | Contents |
|--------|------|------|----------|
| −9,831 | 3 | `BOOT_MAGIC` | 20,261,128: a disk without it is not bootable |
| −9,828 | 4 | `BOOT_FIRST` | the image's first block |
| −9,825 | 5 | `BOOT_COUNT` | how many blocks it has, 1 to 6,561 |
| −9,822 | 6 | `BOOT_PAGE` | the RAM page its first block goes to; block `BOOT_FIRST + i` goes to page `BOOT_PAGE + i` |
| −9,819 | 7 | `BOOT_ENTRY` | the physical address to start it at |
| −9,816 | 8 | | reserved: 0 |

**What the ROM does.** It reads block 0 into page 1, checks the
record, reads the image's blocks into their pages with one command, and
jumps to `BOOT_ENTRY` in ring +1, untranslated, with `a0` = 0, as a
program loaded at address 0 starts. It polls the block device rather
than take its interrupt, so the interrupt controller, the trap CSRs and
the other harts are as reset left them, and the block device has no
command in flight and nothing to acknowledge, and its BLOCK, COUNT and
LIST registers read 0, 1 and 0 again, as at reset. The first words of page 1
are its scratch: block 0 and the scatter list. An image may load over
them, and RAM it does not load stays zero. Other registers hold what
the ROM left in them.

**When it cannot boot**, it prints `boot: ` and why on the console,
and powers off with exit status `BOOT_FAILED` (1):

| Message | Why |
|---------|-----|
| `no disk` | the disk has no blocks |
| `no boot record in block 0` | word 3 of block 0 is not `BOOT_MAGIC` |
| `the boot record is not valid` | `BOOT_COUNT` is below 1 or above 6,561, or `BOOT_PAGE` is not a RAM page |
| `the image does not fit in RAM` | its last page is past the machine's RAM (`--ram-pages`) |
| `reading the disk failed` | a block it reads is not on the disk |

The boot page holds an image that needs nothing of page 0, so the same
image boots this way and loads directly when its pages are given as a
program file, if it also puts something at address 0 that jumps to
`BOOT_ENTRY`. trylobite's disks are built this way.

## The binary floating point coprocessor (page −15)

IEEE 754 binary32 and binary64 arithmetic beside the ternary core, for
programs whose floats are binary, such as Rust's `f32` and `f64`
(`docs/rust.md`). The core's own floats are ternary (`isa.md` 7.11);
binary ones belong to a coprocessor on the bus, as the 8087 sat beside
the 8086, so the instruction set stays ternary. CONTEXT.md gives the
reasoning.

Page −15 is centred on `F = −295245`. Each hart has its own bank of
registers, so harts never see each other's operands. Every register is
one word.

| Address | Register | Access | Behaviour |
|---------|----------|--------|-----------|
| F−12 | PRESENT | ro | `FPU_VERSION`, 1: the coprocessor is here |
| F−9, F−6 | A_LO, A_HI | rw | operand A |
| F−3, F | B_LO, B_HI | rw | operand B |
| F+3, F+6 | C_LO, C_HI | rw | operand C (`FPU_FMA` only) |
| F+9 | OP | wo | computes the operation written from A, B and C into R at once; other values are ignored |
| F+12, F+15 | R_LO, R_HI | rw | the result; writable so that a kernel can restore a bank |

- **Values are bits.** A binary32 value or a 32-bit integer is one
  word, LO, holding the bit pattern as a number from 0 to 2³² − 1; HI
  is not read, and a 32-bit result leaves R_HI 0. A binary64 value or
  a 64-bit integer is two such words, low bits in LO. A word is read
  modulo 2³², so a negative 32-bit integer can also be written as its
  balanced value. This is how `detrytus ll` holds `float`, `double`,
  `i32` and `i64` already.
- **The sign of OP is the precision**: +n works on binary64, −n on
  binary32.

| n | Name | R |
|---|------|---|
| 1 | `FPU_ADD` | A + B |
| 2 | `FPU_SUB` | A − B |
| 3 | `FPU_MUL` | A × B |
| 4 | `FPU_DIV` | A / B |
| 5 | `FPU_SQRT` | √A |
| 6 | `FPU_FMA` | A × B + C, rounded once |
| 7 | `FPU_MIN` | minimumNumber(A, B): a NaN is ignored, −0 is below +0 |
| 8 | `FPU_MAX` | maximumNumber(A, B) |
| 9 | `FPU_CMP` | −1, 0 or +1 as A is below, equal to or above B; `FPU_UNORDERED` (2) when either is a NaN |
| 10 | `FPU_CVT` | A in the other precision: −10 binary32 to binary64, +10 binary64 to binary32 |
| 11, 12 | `FPU_TO_I32`, `FPU_TO_U32` | A as a signed or unsigned 32-bit integer |
| 13, 14 | `FPU_TO_I64`, `FPU_TO_U64` | A as a 64-bit integer |
| 15, 16 | `FPU_FROM_I32`, `FPU_FROM_U32` | the 32-bit integer in A as a float |
| 17, 18 | `FPU_FROM_I64`, `FPU_FROM_U64` | the 64-bit integer in A as a float |

- **Rounding** is to nearest, ties to even, the IEEE default and the
  only mode Rust uses. Conversions to integers truncate toward zero and
  saturate, a NaN giving 0, as Rust's `as` does.
- **NaNs.** A NaN result is the canonical quiet NaN, `0x7fc00000` or
  `0x7ff8000000000000`, as on RISC-V.
- **No flags and no other rounding modes** in this version: nothing
  Rust compiles reads them. They can come later behind a new
  `FPU_VERSION`.
- **Timing.** The result is ready when the OP store completes, so a
  binary64 add is four stores of operands, one of OP and two loads.
- **Context switches.** A bank is state like registers: a kernel that
  lets several processes use the coprocessor saves and restores A, B,
  C and R, eight words, when it switches between them, since a process
  can be interrupted between writing OP and reading R.

## The network device (page −16)

An Ethernet interface that moves whole frames between itself and RAM on
its own, then raises source 3. It is the wire under a network stack:
the kernel's driver sends and receives frames, and everything above
Ethernet (ARP, IP, TCP) is software. CONTEXT.md gives the reasoning.

**A frame is octets, one per tryte.** Each tryte of a frame in RAM holds
one octet, 0 to 255, in wire order: destination address, source address,
type, payload. There is no preamble and no frame check sequence, which
the device adds and checks itself, as real interfaces do. A frame is
`FRAME_MIN` (14, the header) to `FRAME_MAX` (1,514) octets.

Page −16 is centred on `N = −314928`. Every register is one word.

| Address | Register | Access | Behaviour |
|---------|----------|--------|-----------|
| N−27 | PRESENT | ro | `NET_VERSION`, 1: the device is here |
| N−24 | LINK | ro | +1 with a link attached (`--net`), 0 without |
| N−21, N−18 | MAC_HI, MAC_LO | ro | the interface's address, 24 bits each, high half first: `02:00:00:00:00:01` (MAC_HI 131072, MAC_LO 1) unless `--mac` sets another |
| N−15 | DROPPED | ro | frames dropped on arrival since reset (below) |
| N−12 | TX_ADDR | rw | the physical address of the frame to send |
| N−9 | TX_LEN | rw | its length in octets |
| N−6 | TX_COMMAND | wo | `NET_SEND` (+1) sends the frame; ignored while a send is busy, and other values are ignored |
| N−3 | TX_STATUS | ro | `NET_BUSY` (+1), `NET_DONE` (0) or `NET_FAILED` (−1) |
| N | TX_ACK | wo | any write clears the send's finished state |
| N+3 | RX_ADDR | rw | the physical address of a receive buffer |
| N+6 | RX_SIZE | rw | its size in trytes |
| N+9 | RX_COMMAND | wo | `NET_RECEIVE` (+1) gives the buffer to the device for the next frame; ignored while a buffer is given, and other values are ignored |
| N+12 | RX_STATUS | ro | `NET_BUSY` (+1) while the buffer waits for a frame, `NET_DONE` (0) once one is in it, `NET_FAILED` (−1) |
| N+15 | RX_LEN | ro | the length of the frame received |
| N+18 | RX_ACK | wo | any write clears the receive's finished state |

- **Two channels.** Sending and receiving are independent, each with
  its own command, status and finished state, so a driver keeps a
  buffer given for the next frame while it sends.
- **Sending.** TX_COMMAND reads the frame at once, without
  translation, so the buffer may be reused as soon as the command is
  written.
  The send fails, and is finished at once, when there is no link,
  TX_LEN is outside `FRAME_MIN`..`FRAME_MAX`, a tryte of the frame is
  not RAM, or a tryte is not an octet. Otherwise it finishes 100 +
  TX_LEN rounds later, when the frame leaves on the link.
- **Receiving.** RX_COMMAND checks the buffer: if any of its RX_SIZE
  trytes is not RAM, or RX_SIZE is below 1, the receive fails at once.
  The next frame to arrive is then written into it, RX_LEN is set and
  the receive is finished; trytes past the frame are left alone. One
  frame per buffer: the driver gives a buffer again for the next.
- **Waiting frames.** Frames that arrive while no buffer is given wait
  in the device, up to `RX_QUEUE` (9), and fill the next buffers in the
  order they came. A frame that arrives when nine wait, is not a frame
  (shorter than `FRAME_MIN` or longer than `FRAME_MAX`), or is longer
  than the buffer it would fill, is dropped and counted in DROPPED; a
  too-long frame leaves the buffer given for the one after.
- **The link** is looked at for arriving frames every 1,000 rounds
  (when `time` is a multiple of 1,000), so a socket is not asked on
  every step. When every hart waits and a frame could arrive, time
  moves to the next look rather than skipping further.
- **Finished state.** A send or receive that finishes, by success or
  failure, sets its channel's finished state; source 3 is pending while
  either channel's is set. The status keeps the outcome until the
  channel's next command.

**The link.** `run --net loop` attaches a loopback link, which hands
every frame sent straight back, as a cable from the interface to
itself; a run with it stays deterministic. `run --net udp:LOCAL,PEER`
carries each frame as one UDP datagram, sent from the local address to
the peer and received from the peer alone. LOCAL and PEER are
`HOST:PORT`, or a bare port on 127.0.0.1. The framing is QEMU's
(`-netdev dgram` and `-netdev socket,udp=`), so the peer can be another
detrytus, a QEMU guest, or a small bridge to a host TAP device. A
datagram that cannot be sent is lost, as on a wire with nothing at the
other end. Frames come from outside the machine at no particular step,
so a run with a UDP link is not deterministic. `--mac` sets MAC_HI and
MAC_LO, for two machines on one wire.

`run --net user` attaches a link that is a small network: the machine
and a gateway, 10.0.0.1 (MAC 02:00:00:00:00:fe), which the emulator
plays, translating the machine's traffic to the host's own sockets as
QEMU's user networking (slirp) does. The gateway answers ARP for its
address and pings to it. Past it, an ICMP echo request goes out on a
host ping socket (an unprivileged ICMP datagram socket where the host
allows one, else a raw socket, which needs root; with neither, it goes
unanswered); each UDP port of the machine's gets a host UDP socket, and
what comes back to that socket comes back to the port; a TCP
connection becomes a host TCP connection, the emulator ending the
machine's TCP itself: its SYN is answered once the host's connect
succeeds, and reset if that fails. Packets to 10.0.0.1 go to the host's
127.0.0.1, so the machine reaches the host's own servers there. The
machine takes any address on 10.0.0.0/24 (10.0.0.2 by convention); the
gateway answers whichever it uses. A name server, 10.0.0.3 (same
MAC), answers ARP and pings too, and DNS queries on UDP port 53: a
standard query for one name's A records is looked up with the host's
own resolver (its hosts file and DNS, on a thread so the machine never
waits) and answered with every IPv4 address found, TTL 60, or with
NXDOMAIN (rcode 3) if the lookup fails; a query for any other type is
answered with no records. Other traffic to it is dropped. Nothing outside can open a
connection to the machine. The host keeps real time, so a machine on
this link wants `--live`; without it, its idle time is skipped and its
timeouts run out before the host answers.

## Emulator behaviour outside the ISA

- A trap aimed at a ring whose `tvec` is 0 stops the emulator with an
  "unhandled trap" message (exit status 70), naming the cause, `pc` and
  `tval`. Address 0 is the reset vector, so no real handler lives there.
- `run` gives up after `--max-steps` steps (default 10 million, exit
  status 124).
- Features not implemented yet stop the emulator with a message (exit
  status 69): debug triggers.
- An unhandled trap on a hart other than 0 names the hart: "... tval 21
  on hart 1".
- With more than one hart, `run --trace` puts the hart's number first
  on each line.
- The debugger steps a round at a time and looks at one hart, the one
  in focus (`docs/debugger.md`, Several harts).

## Paging

The TLB keeps every successful translation until `fence.vm` removes it
and never evicts one on its own, the strictest behaviour `isa.md` 9.7
allows. A kernel that forgets a `fence.vm` after changing an entry sees
the stale translation at once, rather than on a machine whose TLB
happens to evict sooner. `examples/paging.s` builds a page table and
runs a user program through it.
