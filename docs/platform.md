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
| -3 | hart controller: one slot per hart (proposed, below) |
| -4 | hart controller: this hart's slot, group send, events (proposed) |
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

With several harts (proposed): the harts take one step each per round,
in hart-number order, and `time` advances by one per round, so every
hart reads the same clock and runs stay deterministic. `wfi` skips time
forward only when every running hart is waiting, to the earliest moment
any of them could wake: an enabled timer compare or a scheduled hot-plug
event.

## Harts (proposed)

*Proposed in isa.md 0.5 (5.6); not implemented yet.* The emulator has
one hart until this is built; the layout below is for review.

The reference platform allows up to 729 harts, numbered −364..+364, one
27-tryte slot each in page −3. `--harts N` will choose which are
present at reset (N harts, filled outward from 0 in tree order: 0, then
−1 and +1, then the rest of the middle nine, and so on). Pages −5..−12
are reserved so a later platform can grow past 729.

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
  itself. Reading RECEIVE until it is empty clears it.
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
| C+30 | SPAN | rw, per hart | `k`, 0..6: the subtree is every hart whose number agrees with TARGET except in its lowest `k` trits |
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
  moves its work away and the hart stops itself, after which the
  platform may remove it. The external interrupt (cause −2) of hart
  EVENT_HART is pending while events are queued. The emulator schedules
  hot-plug from the command line or the debugger, at a fixed `time`, so
  runs stay deterministic.

## Emulator behaviour outside the ISA

- A trap aimed at a ring whose `tvec` is 0 stops the emulator with an
  "unhandled trap" message (exit status 70), naming the cause, `pc` and
  `tval`. Address 0 is the reset vector, so no real handler lives there.
- `run` gives up after `--max-steps` steps (default 10 million, exit
  status 124).
- Features not implemented yet stop the emulator with a message (exit
  status 69): floating point, compressed bundles and debug triggers.

## Paging

The TLB keeps every successful translation until `fence.vm` removes it
and never evicts one on its own, the strictest behaviour `isa.md` 9.7
allows. A kernel that forgets a `fence.vm` after changing an entry sees
the stale translation at once, rather than on a machine whose TLB
happens to evict sooner. `examples/paging.s` builds a page table and
runs a user program through it.
