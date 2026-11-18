# Code and data written side by side, placed apart by sections, with the
# console helpers pulled in by .include.

start:
        la    a0, greeting
        call  puts
        la    a0, farewell
        call  puts
        li    a0, 0
        j     halt

        .section data, 2 * 19683        # RAM page 2, placed on first use
greeting:
        .string "code in the boot page, strings in RAM page 2\n"

        .section text                   # back where the code left off
        .include "lib/console.s"

        .section data                   # and the data where it left off
farewell:
        .string "helpers came from lib/console.s\n"
