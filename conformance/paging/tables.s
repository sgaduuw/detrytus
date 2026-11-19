# Constants and base page tables for the paging cases.
#
# One address space, its tables placed statically by .org:
#   root at page ROOT; root entry 0 points to the mid table MID;
#   mid entry 0 points to leaf table LEAF0 (virtual pages -3280..3280).
# A case adds leaf entries with .org at LEAF0 + 3 * page and .word. Mid
# entry 1 (pages SUPER-3280..SUPER+3280) is left to the case: a pointer
# to LEAF1 or a superpage leaf. The assembler refuses to place two
# values at one address, so a case cannot overwrite these by accident.
# Ends with .org 0, so code that follows starts the boot firmware.

        .equ  PAGE,   19683
        .equ  CODE,   10                 # page holding the program under test
        .equ  SUPER,  6561               # pages in a mid-level superpage
        .equ  ROOT,   101
        .equ  MID,    102
        .equ  LEAF0,  103 * PAGE         # leaf tables, as addresses
        .equ  LEAF1,  104 * PAGE
        .equ  MID1,   MID * PAGE + 3     # address of mid entry 1

        # Page-table entry trits (docs/isa.md 9.3), as values.
        .equ  LEAF,   1                  # V = +
        .equ  PTR,    -1                 # V = -
        .equ  R,      3
        .equ  W,      9
        .equ  X,      27
        .equ  U,      81
        .equ  G,      243
        .equ  A,      729
        .equ  D,      2187
        .equ  S,      6561

        .org  ROOT * PAGE
        .word PTR + MID * PAGE
        .org  MID * PAGE
        .word PTR + LEAF0
        .org  0
