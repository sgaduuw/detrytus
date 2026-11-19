# The usual start: base tables, the code page mapped for the user ring
# (r-x, user), and firmware that drops to ring -1 at CODE. The program
# under test follows the include.

        .include "tables.s"
        .equ  PTBR, 1 + ROOT * PAGE      # mode +, ASID 0
        .equ  RING, -1
        .org  LEAF0 + 3 * CODE
        .word LEAF + R + X + U + A + CODE * PAGE
        .org  0
        .include "boot.s"
