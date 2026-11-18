# Console helpers, for .include. Defines nothing that runs on its own,
# so include it after the code that should start at address 0.

        .equ  CONSOLE_OUT, -19683
        .equ  POWER_OFF,   -39366

# puts: print the 0-terminated string at a0. Clobbers t0 and t1.
puts:   li    t1, CONSOLE_OUT
1:      lt    t0, 0(a0)
        beqz  t0, 2f
        sw    t0, 0(t1)
        addi  a0, a0, 1
        j     1b
2:      ret

# halt: power off with exit code a0.
halt:   li    t0, POWER_OFF
        sw    a0, 0(t0)
