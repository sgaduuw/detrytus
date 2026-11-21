# Three harts: the boot hart starts the other two of its triad, each sends
# its hart number to the boot hart's mailbox, and the boot hart greets
# them in the order the messages arrive. Run it with three harts:
#
#     detrytus run --harts 3 examples/harts.s

        .equ  SLOTS, -59049       # hart h's slot is centred on SLOTS + 27*h
        .equ  LOCAL, -78732       # the accessing hart's own slot
        .equ  STATE, -12
        .equ  START_PC, -9
        .equ  CONTROL, -3
        .equ  SEND, 0
        .equ  RECEIVE, 3
        .equ  MAILBOX_EMPTY, -1   # RECEIVE with nothing waiting
        .equ  MAX_HART, 364       # hart numbers run -364..+364; messages
                                  # are never negative, so a hart sends
                                  # its number + MAX_HART

start:
        li    s0, SLOTS + 27
        lw    t0, STATE(s0)
        bltz  t0, alone           # hart +1 is absent
        la    t1, worker
        li    t2, 1
        sw    t1, START_PC(s0)
        sw    t2, CONTROL(s0)
        li    s0, SLOTS - 27
        sw    t1, START_PC(s0)
        sw    t2, CONTROL(s0)

        li    t0, 0t+000          # ie trit 3: the mailbox ends a wfi
        csrw  mie, t0
        li    s1, 2               # greetings still to come
        li    s2, LOCAL
next:   wfi
        lw    s3, RECEIVE(s2)
        bltz  s3, next            # MAILBOX_EMPTY: woken for nothing
        addi  s3, s3, -MAX_HART
        la    a0, hart
        call  puts
        li    t1, CONSOLE_OUT
        li    t0, '+'
        bgtz  s3, 1f
        li    t0, '-'
1:      sw    t0, 0(t1)
        abs   t0, s3
        addi  t0, t0, '0'
        sw    t0, 0(t1)
        la    a0, hello
        call  puts
        addi  s1, s1, -1
        bnez  s1, next
        li    a0, 0
        j     halt

alone:  la    a0, usage
        call  puts
        li    a0, 1
        j     halt

# Each worker sends its hart number to hart 0, then stops.
worker: csrr  t0, hartid
        addi  t0, t0, MAX_HART
        li    t1, SLOTS
        sw    t0, SEND(t1)
        li    t0, -1
        li    t1, LOCAL
        sw    t0, CONTROL(t1)

hart:   .string "hart "
hello:  .string " says hello\n"
usage:  .string "this example needs three harts: run it with --harts 3\n"

        .include "lib/console.s"
