# Start code and runtime for Rust programs translated by detrytus ll,
# on the bare machine, as the tests run them: main(argc, argv) gets one
# argument, "prog", its result powers the machine off as the exit code,
# and write() prints to the console.

        .equ  POWER_OFF, -39366
        .equ  CONSOLE_OUT, -19683

        .section reset, 0
        li    sp, 9 * 19683
        j     _start

        .section program, 2 * 19683
_start: li    a0, 2
        la    a1, argv
        call  main
        li    t0, POWER_OFF
        sw    a0, 0(t0)

# abort(): what a panic does; exit code 134, as SIGABRT gives.
abort:  li    a0, 134
        li    t0, POWER_OFF
        sw    a0, 0(t0)

# write(fd, buf, n): print n bytes at buf; returns n.
write:  li    t0, CONSOLE_OUT
        mv    t1, a1
        add   t2, a1, a2
1:      bge   t1, t2, 2f
        lt    t3, 0(t1)
        sw    t3, 0(t0)
        addi  t1, t1, 1
        j     1b
2:      mv    a0, a2
        ret

# strlen(s)
strlen: mv    t0, a0
1:      lt    t1, 0(t0)
        beqz  t1, 2f
        addi  t0, t0, 1
        j     1b
2:      sub   a0, t0, a0
        ret

# memcpy(dst, src, n) and memmove: returns dst.
memmove:
        blt   a1, a0, 3f
memcpy: mv    t0, a0
        add   t2, a1, a2
1:      bge   a1, t2, 2f
        lt    t1, 0(a1)
        st    t1, 0(t0)
        addi  a1, a1, 1
        addi  t0, t0, 1
        j     1b
2:      ret
3:      add   t0, a0, a2              # backwards, for an overlap above
        add   a1, a1, a2
4:      beq   t0, a0, 2b
        addi  t0, t0, -1
        addi  a1, a1, -1
        lt    t1, 0(a1)
        st    t1, 0(t0)
        j     4b

# memset(dst, byte, n): returns dst.
memset: mv    t0, a0
        add   t2, a0, a2
1:      bge   t0, t2, 2f
        st    a1, 0(t0)
        addi  t0, t0, 1
        j     1b
2:      ret

# memcmp(a, b, n) and bcmp: <0, 0 or >0 as the first difference.
bcmp:
memcmp: add   t2, a0, a2
1:      bge   a0, t2, 2f
        lt    t0, 0(a0)
        lt    t1, 0(a1)
        bne   t0, t1, 3f
        addi  a0, a0, 1
        addi  a1, a1, 1
        j     1b
2:      li    a0, 0
        ret
3:      sub   a0, t0, t1
        ret

# The arguments: pointers as Rust lays them out, 4 bytes each.
argv:   .tryte (prog) % 256, (prog) / 256 % 256, (prog) / 65536 % 256, (prog) / 16777216 % 256
        .tryte (arg1) % 256, (arg1) / 256 % 256, (arg1) / 65536 % 256, (arg1) / 16777216 % 256
prog:   .string "prog"
arg1:   .string " hi there  9 "
