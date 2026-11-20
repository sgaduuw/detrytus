        .equ  POWER_OFF, -39366
        .equ  DATA, 19683
        la    t0, handler
        csrw  mtvec, t0
start:
begin:  li    s0, DATA              # two labels: source order and name order differ
        li    a0, 5
        call  double
        la    t2, double
        jalr  ra, 0(t2)             # a call that is not jal
        sw    a0, 0(s0)
        lw    a1, 0(s0)
        sys
after:  li    t0, POWER_OFF
        sw    a1, 0(t0)
double: add   a0, a0, a0
        ret
handler:
        csrr  t1, mtpc
        addi  t1, t1, 3
        csrw  mtpc, t1
        tret
