# Firmware: enable paging, install a handler, drop to RING at CODE.
#
# Define PTBR and RING before including. The handler leaves mtcause in
# a0 and mtval in a1 and powers off with the cause, so a program that
# ends in sys exits with 4. It runs in ring +1, which is never
# translated, however the page tables look.

        li    t0, PTBR
        csrw  ptbr, t0
        la    t0, handler
        csrw  mtvec, t0
        li    t0, CODE * PAGE
        csrw  mtpc, t0
        li    t0, RING
        csrw  mtstatus, t0
        tret
handler:
        csrr  a0, mtcause
        csrr  a1, mtval
        li    t6, -39366
        sw    a0, 0(t6)

        .org  CODE * PAGE
