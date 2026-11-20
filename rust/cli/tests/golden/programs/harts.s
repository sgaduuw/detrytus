# Hart 0 starts hart 1; each prints a letter, then hart 1 stops itself
# and hart 0 powers off once it has.

        li    s0, -59049 + 27
        la    t0, second
        sw    t0, -9(s0)
        li    t0, 1
        sw    t0, -3(s0)
        li    a0, 'A'
        li    t6, -19683
        sw    a0, 0(t6)
wait:   lw    t0, -12(s0)
        bnez  t0, wait
        li    t6, -39366
        sw    zero, 0(t6)
second: li    a0, 'B'
        li    t6, -19683
        sw    a0, 0(t6)
        li    t0, -1
        li    t1, -78732
        sw    t0, -3(t1)
