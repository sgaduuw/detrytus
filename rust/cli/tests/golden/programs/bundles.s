# Bundles written by hand: two c. instructions share a word.
start:  c.li  a0, 5
        c.addi a0, 1
        c.jal sub
        li    t6, -39366
        sw    a0, 0(t6)
sub:    c.shi a0, 1
        c.jr  ra
