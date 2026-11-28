# Read the real-time clock and print it: seconds since the Unix epoch,
# a dot, and the microseconds. `detrytus run --clock 1800000000
# examples/clock.s` prints the same every run; without --clock the time
# is the host's.

        .equ  CLOCK,       -39360        # power page: seconds since 1970
        .equ  CLOCK_MICROS, -39357       # ... and microseconds past it

        .equ  DIGITS, 19683            # page 1, RAM: putn's digits go below

start:
        li    s0, CLOCK
        lw    a0, 0(s0)          # latches the microseconds for this hart
        lw    s1, 3(s0)          # CLOCK_MICROS: the same moment
        li    a1, 1
        call  putn
        li    a0, '.'
        call  putc
        mv    a0, s1
        li    a1, 6              # six digits, leading zeros kept
        call  putn
        li    a0, '\n'
        call  putc
        li    a0, 0
        j     halt

# putn: print a0, not negative, in at least a1 decimal digits.
# Clobbers t0 .. t4.
putn:   li    t0, 10
        li    t4, DIGITS
        mv    t1, t4             # digits go below DIGITS, last first
1:      rem   t2, a0, t0
        addi  t2, t2, '0'
        addi  t1, t1, -1
        st    t2, 0(t1)
        div   a0, a0, t0
        addi  a1, a1, -1
        bnez  a0, 1b
        bgtz  a1, 1b
        li    t3, CONSOLE_OUT
2:      lt    t2, 0(t1)
        sw    t2, 0(t3)
        addi  t1, t1, 1
        bne   t1, t4, 2b
        ret

putc:   li    t0, CONSOLE_OUT
        sw    a0, 0(t0)
        ret

        .include "lib/console.s"
