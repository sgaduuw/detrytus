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

A page is 19,683 trytes centred on a multiple of 19,683, so page `p`
spans `p·19683 − 9841` .. `p·19683 + 9841`. The loader may place data in
RAM pages too; only the program writes are checked.

## Devices

| Address | Register | Behaviour |
|---------|----------|-----------|
| −19683 | console out | writing a Unicode code point prints it |
| −19680 | console in | reading returns the next input code point, or −1 if none |
| −39366 | power off | writing any value stops the machine; the value becomes the exit code |

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
| S | SEND | wo | queues the word in hart `h`'s mailbox |
| S+3 | RECEIVE | ro | own slot only: takes the oldest message, 0 if empty |
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
  running and its mailbox has room; otherwise it is rejected and the
  word is dropped. The result of the hart's last send, to any slot or
  group, is in LAST (below), so a sender checks it when delivery
  matters; a send never fails silently.
- **Doorbell.** Hart `h`'s software interrupt (cause −3, `ip` trit 3)
  is pending while COUNT > 0, ORed with the bit software can set
  itself. Reading RECEIVE until it is empty clears it. It appears in
  ring +1's `ip`, or ring 0's when `ideleg` trit 3 is `+`.
- **Own slot only.** RECEIVE and CONTROL −1 act only from the hart that
  owns the slot; from another hart a read of RECEIVE gives 0 and takes
  nothing.
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
| C+33 | GSEND | wo | sends the word to every running hart in the subtree except the sender |
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

## Emulator behaviour outside the ISA

- A trap aimed at a ring whose `tvec` is 0 stops the emulator with an
  "unhandled trap" message (exit status 70), naming the cause, `pc` and
  `tval`. Address 0 is the reset vector, so no real handler lives there.
- `run` gives up after `--max-steps` steps (default 10 million, exit
  status 124).
- Features not implemented yet stop the emulator with a message (exit
  status 69): floating point, compressed bundles and debug triggers.
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
