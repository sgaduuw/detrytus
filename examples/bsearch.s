# Binary search with a three-way branch: docs/isa.md section 10.
# Looks up a few keys and prints each index (or -1) on the console.

        .equ  CONSOLE_OUT, -19683
        .equ  POWER_OFF,   -39366
        .equ  STACK_TOP,   19683 * 3     # centre of RAM page 3

start:
        li    sp, STACK_TOP
        la    s0, keys
        li    s1, 5                       # number of keys
next:
        beqz  s1, done
        la    a0, table
        li    a1, 9
        lw    a2, 0(s0)
        call  bsearch
        call  print_number
        li    a0, 10                      # newline
        call  putc
        addi  s0, s0, 3
        addi  s1, s1, -1
        j     next
done:
        li    t0, POWER_OFF
        sw    zero, 0(t0)

# a0 = address of a sorted array of words, a1 = length, a2 = key.
# Returns the index of key in a0, or -1.
bsearch:
        li    t0, 0              # lo
        addi  t1, a1, -1         # hi
        li    t3, 2
loop:
        bgt   t0, t1, notfound
        add   t2, t0, t1
        div   t2, t2, t3         # mid: halving is a real division here
        shi   t4, t2, 1          # word index to tryte offset: x3 is one shift
        add   t4, t4, a0
        lw    t5, 0(t4)
        cmp   t6, a2, t5         # sign(key - a[mid])
        br3   t6, left, right    # -: go left, +: go right, 0: found
        mv    a0, t2
        ret
left:   addi  t1, t2, -1
        j     loop
right:  addi  t0, t2, 1
        j     loop
notfound:
        li    a0, -1
        ret

# Print a0 in decimal. Uses the stack for the digits.
print_number:
        addi  sp, sp, -3
        sw    ra, 0(sp)
        bgez  a0, positive
        mv    t0, a0
        li    a0, '-'
        sw    t0, -3(sp)
        call  putc
        lw    a0, -3(sp)
        neg   a0, a0
positive:
        li    t1, 10
        addi  t2, sp, -1         # digits go below the saved ra, which
                                 # covers sp-1 .. sp+1
digits:
        rem   t3, a0, t1
        addi  t3, t3, '0'
        addi  t2, t2, -1
        st    t3, 0(t2)
        div   a0, a0, t1
        bnez  a0, digits
emit:
        lt    a0, 0(t2)
        call  putc_keep
        addi  t2, t2, 1
        addi  t4, sp, -1
        bne   t2, t4, emit
        lw    ra, 0(sp)
        addi  sp, sp, 3
        ret

putc:                            # print the code point in a0
putc_keep:                       # (clobbers only t5)
        li    t5, CONSOLE_OUT
        sw    a0, 0(t5)
        ret

table:  .word -40, -7, 0, 3, 9, 27, 81, 100, 243
keys:   .word 27, -40, 243, 5, 0
