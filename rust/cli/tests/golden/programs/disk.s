# Reads block 0 into page 2. If its first word is set, powers off with
# it; if not, sets it to 42 in page 1, writes page 1 to block 0, and
# powers off with 0. Run twice on one disk: 0, then 42.
        .equ  BLK, -275562
        li    s1, BLK
        la    t0, two
        sw    t0, 0(s1)           # LIST
        li    t0, 1
        sw    t0, 3(s1)           # COMMAND: read
1:      lw    t0, 6(s1)           # STATUS
        bgtz  t0, 1b           # until not busy
        li    t1, 39366           # page 2
        lw    a0, -9840(t1)
        bnez  a0, done
        li    t1, 19683           # page 1
        li    t0, 42
        sw    t0, -9840(t1)
        la    t0, one
        sw    t0, 0(s1)
        li    t0, -1
        sw    t0, 3(s1)           # COMMAND: write
2:      lw    t0, 6(s1)
        bgtz  t0, 2b
        li    a0, 0
done:   li    t6, -39366
        sw    a0, 0(t6)
one:    .word 1
two:    .word 2
