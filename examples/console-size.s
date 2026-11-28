# Print the console's size as `stty size` does: rows, a space, columns.
# `detrytus run --console-size 24x80 examples/console-size.s` prints
# 24 80; run at a terminal it prints the terminal's, and with neither,
# 0 0 (not known).

        .equ  CONSOLE_ROWS,    -19677    # console page: the terminal's rows
        .equ  CONSOLE_COLUMNS, -19674    # ... and columns

        .equ  DIGITS, 19683            # page 1, RAM: putn's digits go below

start:
        li    s0, CONSOLE_ROWS
        lw    a0, 0(s0)
        call  putn
        li    a0, ' '
        call  putc
        lw    a0, 3(s0)          # CONSOLE_COLUMNS
        call  putn
        li    a0, '\n'
        call  putc
        li    a0, 0
        j     halt

# putn: print a0, not negative, in decimal. Clobbers t0 .. t4.
putn:   li    t0, 10
        li    t4, DIGITS
        mv    t1, t4             # digits go below DIGITS, last first
1:      rem   t2, a0, t0
        addi  t2, t2, '0'
        addi  t1, t1, -1
        st    t2, 0(t1)
        div   a0, a0, t0
        bnez  a0, 1b
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
