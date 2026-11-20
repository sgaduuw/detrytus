# Loaded with second.s: both define start, at different addresses.
start:  li    a0, 1
only_first:
        li    t6, -39366
        sw    a0, 0(t6)
