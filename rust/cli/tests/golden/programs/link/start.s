# Start code for main.tc and sum.tc, linked from objects: main returns
# the exit code.
        li    sp, 4 * 19683
        call  main
        li    t0, -39366
        sw    a0, 0(t0)
