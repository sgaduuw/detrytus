# bare.s with a global pointer: the same start, but gp holds the
# program's address and the section says so with .gp, so the compiler
# loads and stores global scalars by name (tests/programs.rs runs each
# program both ways).

        .equ  POWER_OFF, -39366
        .equ  CONSOLE_OUT, -19683

        li    sp, 4 * 19683
        li    gp, 2 * 19683
        j     _start

        .section program, 2 * 19683
        .gp   2 * 19683
_start: call  main
        li    t0, POWER_OFF
        sw    a0, 0(t0)

# putc(int c): print code point c on the console.
putc:   li    t0, CONSOLE_OUT
        sw    a0, 0(t0)
        ret
