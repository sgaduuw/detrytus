# Print a greeting on the console, then power off.

        .equ  CONSOLE_OUT, -19683
        .equ  POWER_OFF,   -39366

start:
        la    t0, message
        li    t1, CONSOLE_OUT
loop:
        lt    t2, 0(t0)          # one code point per tryte
        beqz  t2, done
        sw    t2, 0(t1)
        addi  t0, t0, 1
        j     loop
done:
        li    t1, POWER_OFF
        sw    zero, 0(t1)

message:
        .string "Hello from a balanced-ternary machine!\n"
