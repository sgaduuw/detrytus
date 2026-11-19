# As user.s, but the code page is a kernel page and firmware drops to
# ring 0.

        .include "tables.s"
        .equ  PTBR, 1 + ROOT * PAGE
        .equ  RING, 0
        .org  LEAF0 + 3 * CODE
        .word LEAF + R + X + A + CODE * PAGE
        .org  0
        .include "boot.s"
