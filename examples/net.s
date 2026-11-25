# Send an Ethernet frame and read it back: run with --net loop, a link
# that hands every frame sent straight back (docs/platform.md, The
# network device). A frame is octets, one per tryte.

        .equ  NET,         -314928
        .equ  MAC_HI,      -21
        .equ  MAC_LO,      -18
        .equ  TX_ADDR,     -12
        .equ  TX_LEN,      -9
        .equ  TX_COMMAND,  -6
        .equ  TX_STATUS,   -3
        .equ  RX_ADDR,     3
        .equ  RX_SIZE,     6
        .equ  RX_COMMAND,  9
        .equ  RX_STATUS,   12
        .equ  RX_LEN,      15
        .equ  NET_SEND,    1
        .equ  NET_RECEIVE, 1
        .equ  FRAME_MAX,   1514
        .equ  HEADER,      14
        .equ  CONSOLE_OUT, -19683
        .equ  POWER_OFF,   -39366
        .equ  FRAME,       19683          # RAM page 1
        .equ  BUFFER,      39366          # RAM page 2

start:
        li    s1, NET
        li    s2, FRAME
        li    t0, 255                     # destination: everyone
        li    t1, 6
1:      st    t0, 0(s2)
        addi  s2, s2, 1
        addi  t1, t1, -1
        bnez  t1, 1b
        lw    a0, MAC_HI(s1)              # source: our address, high
        call  octets3                     # octets first
        lw    a0, MAC_LO(s1)
        call  octets3
        li    t0, 0x88                    # type 88B5: local experiment
        st    t0, 0(s2)
        li    t0, 0xB5
        st    t0, 1(s2)
        addi  s2, s2, 2
        la    t0, message                 # the payload
2:      lt    t1, 0(t0)
        beqz  t1, 3f
        st    t1, 0(s2)
        addi  s2, s2, 1
        addi  t0, t0, 1
        j     2b

3:      li    t0, BUFFER                  # a buffer for what comes back
        sw    t0, RX_ADDR(s1)
        li    t0, FRAME_MAX
        sw    t0, RX_SIZE(s1)
        li    t0, NET_RECEIVE
        sw    t0, RX_COMMAND(s1)
        li    t0, FRAME
        sw    t0, TX_ADDR(s1)
        sub   t1, s2, t0
        sw    t1, TX_LEN(s1)
        li    t0, NET_SEND
        sw    t0, TX_COMMAND(s1)
        lw    t0, TX_STATUS(s1)           # no link: failed at once
        bltz  t0, fail
4:      lw    t0, RX_STATUS(s1)           # busy until a frame is in
        bgtz  t0, 4b
        bltz  t0, fail

        li    t0, BUFFER + HEADER         # print the payload
        lw    t1, RX_LEN(s1)
        addi  t1, t1, -HEADER
        li    t2, CONSOLE_OUT
5:      lt    t3, 0(t0)
        sw    t3, 0(t2)
        addi  t0, t0, 1
        addi  t1, t1, -1
        bnez  t1, 5b
        li    a0, 0
        j     off
fail:   la    a0, nolink
        li    t2, CONSOLE_OUT
6:      lt    t3, 0(a0)
        beqz  t3, 7f
        sw    t3, 0(t2)
        addi  a0, a0, 1
        j     6b
7:      li    a0, 1
off:    li    t1, POWER_OFF
        sw    a0, 0(t1)

# Store the 24-bit number in a0 as three octets at s2, high first, and
# move s2 past them.
octets3:
        li    t1, 256
        rem   t0, a0, t1
        st    t0, 2(s2)
        div   a0, a0, t1
        rem   t0, a0, t1
        st    t0, 1(s2)
        div   a0, a0, t1
        st    a0, 0(s2)
        addi  s2, s2, 3
        ret

message:
        .string "Hello over the wire!\n"
nolink:
        .string "no link: run with --net loop\n"
