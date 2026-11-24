# Start code for Trice programs on the bare machine, with no operating
# system, as the tests run them: reset enters here, the program runs
# from RAM page 2 with its stack below page 4, and main's result
# powers the machine off as the exit code.

        .equ  POWER_OFF, -39366
        .equ  CONSOLE_OUT, -19683

        # Reset code gets a section of its own, so that code linked in
        # from other objects (in text) follows program, in RAM.
        .section reset, 0
        li    sp, 4 * 19683
        j     _start

        .section program, 2 * 19683
_start: call  main
        li    t0, POWER_OFF
        sw    a0, 0(t0)

# putc(int c): print code point c on the console. Global, for programs
# linked from objects (docs/linker.md).
        .global putc
putc:   li    t0, CONSOLE_OUT
        sw    a0, 0(t0)
        ret
