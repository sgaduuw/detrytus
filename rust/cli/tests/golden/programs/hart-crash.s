# Hart 1 runs into a breakpoint with no handler.
        li    s0, -59049 + 27
        la    t0, second
        sw    t0, -9(s0)
        li    t0, 1
        sw    t0, -3(s0)
spin:   j     spin
second: brk
