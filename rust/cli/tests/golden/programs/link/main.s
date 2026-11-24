# A program in two objects: this one starts at 0 and calls lib.s.
        .global start
start:  la    a0, greeting
        call  puts
        la    t0, answer
        lw    a0, 0(t0)
        call  halt
        .section data
greeting: .string "linked\n"
