# Hart 0 starts hart 1 and spins; hart 1 installs a trap handler, stores
# to RAM and runs into a breakpoint, whose handler powers off.
        .equ  DATA, 19683
start:  li    s0, -59049 + 27
        la    t0, second
        sw    t0, -9(s0)
        li    t0, 1
        sw    t0, -3(s0)
spin:   j     spin
second: la    t0, handler
        csrw  mtvec, t0
        li    t0, DATA
        li    t1, 5
        sw    t1, 0(t0)
        brk
handler:
        li    t6, -39366
        sw    zero, 0(t6)
