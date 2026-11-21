# Copy the console's input to its output, capitalising ASCII letters,
# then power off. Feed it text with `detrytus run --input FILE`.
#
# Console in reads a code point, which is never negative, or a negative
# value once no input is left. One three-way branch tells the cases
# apart: negative ends the program, zero (NUL) is skipped, positive is
# a character. Test the sign, never a particular negative number.

        .equ  CONSOLE_OUT, -19683
        .equ  CONSOLE_IN,  -19680
        .equ  POWER_OFF,   -39366

start:
        li    t0, CONSOLE_IN
        li    t1, CONSOLE_OUT
loop:
        lw    t2, 0(t0)
        br3   t2, done, char     # zero falls through: skip it
        j     loop
char:
        li    t3, 'a'
        blt   t2, t3, put
        li    t3, 'z'
        bgt   t2, t3, put
        addi  t2, t2, 'A' - 'a'
put:
        sw    t2, 0(t1)
        j     loop
done:
        li    t1, POWER_OFF
        sw    zero, 0(t1)
