# Includes resolve against the including file's directory.
start:  li    a0, 0
parts:  .include "lib/part.s"
        li    t6, -39366
        sw    a0, 0(t6)
