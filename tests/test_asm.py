import pytest

from detrytus import isa as I
from detrytus.asm import AsmError, assemble


def words(image):
    return [(e.addr, e.text) for e in image.listing if e.word is not None]


def test_first_word_is_centred_on_zero():
    img = assemble("nop")
    assert words(img) == [(0, "addi zero, zero, 0")]
    # A centred word covers a-1, a, a+1.
    assert set(img.trytes) == {-1, 0, 1}


def test_trytes_then_words_keep_words_aligned():
    img = assemble(
        """
        .org 30
        a: .tryte 5
        b: nop
        """
    )
    assert img.symbols == {"a": 29, "b": 33}
    assert img.trytes[29] == 5


def test_org_must_be_word_aligned():
    with pytest.raises(AsmError, match="word-aligned"):
        assemble(".org 31")


def test_align_centres_on_a_power_of_three():
    img = assemble("nop\n.align 3\nhere: nop")
    assert img.symbols["here"] == 27


def test_li_picks_short_or_long_form():
    short = assemble("li a0, 21523360")
    long = assemble("li a0, 21523361")
    assert len(words(short)) == 1
    assert len(words(long)) == 2


def test_li_with_a_forward_reference_reserves_the_long_form():
    img = assemble("li a0, later\nlater: nop")
    assert [t for _, t in words(img)][:2] == ["lui a0, 0", "addi a0, a0, 6"]
    assert img.symbols["later"] == 6


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


def test_tmap_builds_a_unary_table():
    # f maps -,0,+ to +,0,+ : the trit magnitude, which is tabs.
    img = assemble("tmap a0, a1, 0t+0+")
    assert img.listing[0].word == I.encode(
        I.OPS["tlut"], rd=1, rs1=2, fn=I.make_table(lambda a, b: abs(a))
    )


def test_strings_store_one_code_point_per_tryte():
    img = assemble('.org 3\ns: .string "Hé\\n"')
    assert [img.trytes[2 + i] for i in range(4)] == [ord("H"), ord("é"), 10, 0]


@pytest.mark.parametrize(
    ("source", "message"),
    [
        ("add a0, a1", "takes 3 operand"),
        ("frob a0", "unknown instruction"),
        ("x: nop\nx: nop", "defined twice"),
        ("addi a0, a0, nowhere", "undefined symbol"),
        ("add a0, a1, q9", "not a register"),
        (".tryte 9842", "does not fit a tryte"),
        ("lw a0, 4", "expected imm"),
        ("x: .tryte 1\nj x", "not word aligned"),
    ],
)
def test_errors_name_the_problem_and_line(source, message):
    with pytest.raises(AsmError, match=message):
        assemble(source)


def test_error_reports_the_line_number():
    with pytest.raises(AsmError, match="line 3"):
        assemble("nop\nnop\nfrob")
