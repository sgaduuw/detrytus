# Automatic pairing: neighbours under .compress that fit a slot share a
# word, a label starts a new one, and what does not fit stays full.
        li    a1, 2
        .compress
        li    a0, 0
        li    t1, 0
loop:   addi  a0, a0, 1
        addi  a0, a0, 1
        addi  a1, a1, -1
        bne   a1, t1, loop
        lui   t2, 1
        mv    a2, a0
        .nocompress
        li    t6, -39366
        sw    a0, 0(t6)
