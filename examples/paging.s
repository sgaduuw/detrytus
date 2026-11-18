# Paged virtual memory: firmware builds a page table, then runs a user
# program that sees only what the table maps. The program prints through
# the console mapped into its own address space, then makes a system call.
#
#   virtual page 1  ->  physical page 5   user code and data (r-x, user)
#   virtual page 2  ->  physical page -1  the console device (rw-, user)

        .equ  PAGE,       19683
        .equ  ROOT_PAGE,  2              # root table at this page's centre
        .equ  MID_PAGE,   3
        .equ  LEAF_PAGE,  4
        .equ  USER_PAGE,  5              # physical home of the user program

        # Page-table entry trits (docs/isa.md 9.3), as values.
        .equ  LEAF,  1                   # V = +
        .equ  PTR,   -1                  # V = -
        .equ  R,     3
        .equ  W,     9
        .equ  X,     27
        .equ  U,     81
        .equ  A,     729
        .equ  D,     2187

        .equ  CONSOLE_OUT, -19683        # physical, for the firmware
        .equ  POWER_OFF,   -39366

# --- firmware: ring +1, never translated -------------------------------------

start:
        # Virtual pages 1 and 2 share root entry 0 and mid entry 0, so one
        # pointer at each upper level reaches the leaf table.
        li    t0, ROOT_PAGE * PAGE
        li    t1, PTR + MID_PAGE * PAGE
        sw    t1, 0(t0)
        li    t0, MID_PAGE * PAGE
        li    t1, PTR + LEAF_PAGE * PAGE
        sw    t1, 0(t0)
        li    t0, LEAF_PAGE * PAGE
        li    t1, LEAF + R + X + U + A + USER_PAGE * PAGE
        sw    t1, 3(t0)                  # entry 1: virtual page 1
        li    t1, LEAF + R + W + U + A + D - PAGE
        sw    t1, 6(t0)                  # entry 2: virtual page 2 -> page -1

        li    t0, 1 + ROOT_PAGE * PAGE   # ptbr: mode +, ASID 0, root page
        csrw  ptbr, t0
        la    t0, trap
        csrw  mtvec, t0
        li    t0, PAGE                   # user entry: centre of virtual page 1
        csrw  mtpc, t0
        li    t0, -1                     # tret drops to ring -1
        csrw  mtstatus, t0
        tret

trap:
        csrr  s0, mtcause
        li    t0, 4
        bne   s0, t0, failed
        la    a0, back
        call  puts
        li    t0, POWER_OFF
        sw    zero, 0(t0)
failed:
        li    t0, POWER_OFF
        sw    s0, 0(t0)                  # exit with the unexpected cause

puts:   li    t1, CONSOLE_OUT
puts_next:
        lt    t2, 0(a0)
        beqz  t2, puts_done
        sw    t2, 0(t1)
        addi  a0, a0, 1
        j     puts_next
puts_done:
        ret

back:   .string "firmware: system call from ring -1\n"

# --- the user program: ring -1, virtual page 1 -------------------------------

        .org  USER_PAGE * PAGE
user:
        la    t0, hello                  # pc-relative, so virtual addresses
        li    t1, 2 * PAGE               # the console, as this program sees it
next:   lt    t2, 0(t0)
        beqz  t2, done
        sw    t2, 0(t1)
        addi  t0, t0, 1
        j     next
done:   sys

hello:  .string "user: hello from virtual page 1\n"
