# Console helpers as an object: puts, halt and a constant for others.
        .global puts, halt, answer
        .equ  CONSOLE_OUT, -19683
        .equ  POWER_OFF,   -39366
        .compress
puts:   li    t1, CONSOLE_OUT
1:      lt    t0, 0(a0)
        beqz  t0, 2f
        sw    t0, 0(t1)
        addi  a0, a0, 1
        j     1b
2:      ret
halt:   li    t0, POWER_OFF
        sw    a0, 0(t0)
        .section data
answer: .word 42
