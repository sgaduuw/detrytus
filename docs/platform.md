# Emulator platform

What the emulator puts around the CPU. The ISA leaves these choices to
the platform (`isa.md` 8.5 and 12.3).

## Memory map

| Physical page | Contents |
|---------------|----------|
| 0 | boot page: the program is loaded here and runs from address 0; read and execute only |
| 1 .. 729 | RAM (`--ram-pages` changes the count) |
| -1 | console |
| -2 | power control |
| -3 | hart controller: one slot per hart (Harts, below) |
| -4 | hart controller: this hart's slot, group send, events |
| -5 .. -12 | reserved for more harts |
| -13 | interrupt controller (Device interrupts, below) |
| -14 | block device (The block device, below) |

A page is 19,683 trytes centred on a multiple of 19,683, so page `p`
spans `p·19683 − 9841` .. `p·19683 + 9841`. The loader may place data in
RAM pages too; only the program writes are checked.

## Devices

| Address | Register | Behaviour |
|---------|----------|-----------|
| −19683 | console out | writing a Unicode code point prints it |
| −19680 | console in | reading returns the next input code point, or a negative state when there is none (Console input states, below; test the sign: a code point is never negative) |
| −39366 | power off | writing any value stops the machine; the value becomes the exit code |

Console input is the text `--input FILE` names, queued before the
first step, plus whatever the debugger's `input` command adds. Once it
is used up, reads say whether more may come (Console input states,
below). A run's input is fixed
in advance, so runs stay deterministic and nothing waits for a person
to type.

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

Page −13 is centred on `I = −255879`. Source `s` has a three-word group
centred on `I + 9·s`; group 0, at `I`, holds the per-hart registers.
Every register is one word, centred on the address given.

| Address | Register | Access | Behaviour |
|---------|----------|--------|-----------|
| I−3 | CLAIM | ro, per hart | takes the lowest-numbered source that is pending, enabled, routed to the reading hart and not in service; marks it in service and returns its number, or 0 if there is none |
| I | COMPLETE | wo, per hart | ends service of the source written; if it is still pending it can be claimed again. Writing a source not in service does nothing |
| I+3 | SOURCES | ro | the highest source number on this platform: 2 |
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
  the start without `--input`, since nothing can type into a run.
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

## Emulator behaviour outside the ISA

- A trap aimed at a ring whose `tvec` is 0 stops the emulator with an
  "unhandled trap" message (exit status 70), naming the cause, `pc` and
  `tval`. Address 0 is the reset vector, so no real handler lives there.
- `run` gives up after `--max-steps` steps (default 10 million, exit
  status 124).
- Features not implemented yet stop the emulator with a message (exit
  status 69): floating point and debug triggers.
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
