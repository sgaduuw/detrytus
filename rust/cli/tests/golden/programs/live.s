# Copies console input to output until the console closes, waiting
# while it is only empty: with --live it echoes stdin whenever it comes.
        .equ  CONSOLE_OUT, -19683
        .equ  CONSOLE_IN,  -19680
        li    t0, CONSOLE_IN
        li    t1, CONSOLE_OUT
1:      lw    t2, 0(t0)
        bgez  t2, 2f
        addi  t3, t2, 1           # -1, empty: 0
        beqz  t3, 1b              # more may come
        li    t6, -39366          # closed
        sw    zero, 0(t6)
2:      sw    t2, 0(t1)
        j     1b
