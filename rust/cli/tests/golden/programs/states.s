# Reads console in twice and powers off with the second read: what the
# console says once its input is used up.
        li    t0, -19680
        lw    a1, 0(t0)
        lw    a0, 0(t0)
        li    t6, -39366
        sw    a0, 0(t6)
