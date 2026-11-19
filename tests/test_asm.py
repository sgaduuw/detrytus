from detrytus.asm import assemble


def words(image):
    return [(e.addr, e.text) for e in image.listing if e.word is not None]


def test_first_word_is_centred_on_zero():
    img = assemble("nop")
    assert words(img) == [(0, "addi zero, zero, 0")]
    # A centred word covers a-1, a, a+1.
    assert set(img.trytes) == {-1, 0, 1}


def test_branch_targets_become_word_offsets():
    img = assemble("top: nop\nbeq a0, a1, top\nbr3 a0, top, end\nend: nop")
    texts = [t for _, t in words(img)]
    assert texts[1] == "beq a0, a1, 0"
    assert texts[2] == "br3.z a0, 0, 9"


def test_suffixes_set_hints_and_ordering():
    img = assemble(
        "x: blt.unlikely a0, a1, x\nbr3.p a0, x, x\namoadd.aqrl a0, a1, (a2)"
    )
    texts = [t for _, t in words(img)]
    assert texts == [
        "blt.unlikely a0, a1, 0",
        "br3.p a0, 0, 0",
        "amoadd.aqrl a0, a1, (a2)",
    ]


def test_named_tables_and_csrs():
    img = assemble("tmin a0, a1, a2\ncsrr t0, mtcause\ncsrw mtvec, t1")
    texts = [t for _, t in words(img)]
    assert texts == [
        "tlut a0, a1, a2, tmin",
        "csrrw t0, mtcause, zero",
        "csrrw zero, mtvec, t1",
    ]
