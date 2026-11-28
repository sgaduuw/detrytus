# The boot ROM: what page 0 holds when `detrytus run` or `detrytus debug`
# is given a disk and no program (docs/platform.md, Booting from a disk).
#
# It reads block 0, finds the boot record in its words 3 to 8, reads the
# blocks the record names into consecutive RAM pages with one command,
# and jumps to the record's entry address in ring +1, untranslated, with
# a0 = 0, as a program loaded at address 0 starts. A disk it cannot boot
# from gets a line on the console saying why, and power off with exit
# status BOOT_FAILED.
#
# It polls the block device rather than take its interrupt, so it leaves
# the interrupt controller, the trap CSRs and other harts as reset left
# them, and puts the block device's registers back as reset left them. Page 1's first words are its scratch: block 0 lands in page 1,
# and the scatter list is built there; the image may load over both.

        .equ  PAGE,         19683
        .equ  CONSOLE_OUT,  -19683
        .equ  POWER_OFF,    -39366
        .equ  RAM_SIZE,     -39363       # how many RAM pages: 1 to N

        .equ  DISK,         -275562      # the block device's centre
        .equ  D_BLOCKS,     -9
        .equ  D_BLOCK,      -6
        .equ  D_COUNT,      -3
        .equ  D_LIST,       0
        .equ  D_COMMAND,    3
        .equ  D_STATUS,     6
        .equ  D_ACK,        9
        .equ  BLOCK_READ,   1
        .equ  BLOCK_BUSY,   1

        .equ  SCRATCH,      PAGE - 9840  # page 1's first word
        .equ  RECORD,       PAGE - 9831  # block 0's word 3, once in page 1
        .equ  R_MAGIC,      0            # the record's words, from RECORD
        .equ  R_FIRST,      3            # the image's first block
        .equ  R_COUNT,      6            # ... how many blocks it has
        .equ  R_PAGE,       9            # ... the RAM page the first goes to
        .equ  R_ENTRY,      12           # ... and where to start it
        .equ  BOOT_MAGIC,   20261128     # a disk without it is not bootable
        .equ  LIST_MAX,     6561         # list words page 1 holds: a page of words
        .equ  BOOT_FAILED,  1            # the exit status when it cannot boot

boot:
        li    s0, DISK
        li    s1, SCRATCH
        lw    t0, D_BLOCKS(s0)
        la    a0, no_disk
        blez  t0, fail

        li    t0, 1                      # block 0 into page 1
        sw    t0, 0(s1)
        li    a0, 0
        li    a1, 1
        call  read

        li    s2, RECORD
        lw    t0, R_MAGIC(s2)
        li    t1, BOOT_MAGIC
        la    a0, no_record
        bne   t0, t1, fail
        lw    s3, R_FIRST(s2)
        lw    s4, R_COUNT(s2)
        lw    s5, R_PAGE(s2)
        lw    s6, R_ENTRY(s2)
        la    a0, bad_record
        blez  s4, fail                   # at least one block,
        li    t0, LIST_MAX
        bgt   s4, t0, fail               # no more than the list holds,
        blez  s5, fail                   # into RAM pages only,
        add   t0, s5, s4
        addi  t0, t0, -1                 # the last page
        li    t1, RAM_SIZE
        lw    t1, 0(t1)
        la    a0, too_big
        bgt   t0, t1, fail               # all of them on this machine

        mv    t0, s1                     # the list: pages s5, s5 + 1, ...
        mv    t1, s5
        mv    t2, s4
1:      sw    t1, 0(t0)
        addi  t0, t0, 3
        addi  t1, t1, 1
        addi  t2, t2, -1
        bgtz  t2, 1b
        mv    a0, s3
        mv    a1, s4
        call  read

        sw    zero, D_BLOCK(s0)          # the block device's registers as
        li    t0, 1                      # reset left them: BLOCK 0, COUNT 1,
        sw    t0, D_COUNT(s0)            # LIST 0
        sw    zero, D_LIST(s0)
        li    a0, 0
        jalr  zero, 0(s6)

# read: blocks a0 .. a0 + a1 - 1 into the pages the list at s1 names,
# waiting for the command to finish. Returns only if it succeeded.
read:
        sw    a0, D_BLOCK(s0)
        sw    a1, D_COUNT(s0)
        sw    s1, D_LIST(s0)
        li    t0, BLOCK_READ
        sw    t0, D_COMMAND(s0)
        li    t1, BLOCK_BUSY
1:      lw    t0, D_STATUS(s0)
        beq   t0, t1, 1b
        sw    zero, D_ACK(s0)
        la    a0, read_failed
        bnez  t0, fail                   # BLOCK_FAILED
        ret

# fail: print "boot: ", the message at a0 and a newline, then power off.
fail:
        mv    s2, a0
        la    a0, prefix
        call  puts
        mv    a0, s2
        call  puts
        li    t0, CONSOLE_OUT
        li    t1, 10
        sw    t1, 0(t0)
        li    t0, POWER_OFF
        li    t1, BOOT_FAILED
        sw    t1, 0(t0)

puts:
        li    t1, CONSOLE_OUT
1:      lt    t2, 0(a0)
        beqz  t2, 2f
        sw    t2, 0(t1)
        addi  a0, a0, 1
        j     1b
2:      ret

prefix:      .string "boot: "
no_disk:     .string "no disk"
read_failed: .string "reading the disk failed"
no_record:   .string "no boot record in block 0"
bad_record:  .string "the boot record is not valid"
too_big:     .string "the image does not fit in RAM"
