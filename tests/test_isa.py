import pytest

from detrytus import isa as I
from detrytus.disasm import disassemble


def test_zero_word_is_illegal():
    with pytest.raises(I.IllegalInstruction):
        I.decode(0)


def test_opcodes_are_unique_and_in_range():
    assert len({op.opcode for op in I.OPS.values()}) == len(I.OPS)
    assert all(-121 <= op.opcode <= 121 and op.opcode for op in I.OPS.values())


def test_register_fields_share_positions_across_formats():
    # Section 6: rs1 sits at the same trits in every format that has it.
    rs1 = {I.FORMAT_FIELDS[f]["rs1"] for f in "RISBT"}
    assert rs1 == {I.RS1}


def test_formats_fill_exactly_27_trits():
    for fmt, fields in I.FORMAT_FIELDS.items():
        widths = sum(w for _, w in fields.values()) + 5
        assert widths == 27, fmt


@pytest.mark.parametrize(
    ("name", "fields"),
    [
        ("add", {"rd": 3, "rs1": -4, "rs2": 13}),
        ("sel", {"rd": 1, "rs1": 2, "rs2": 3, "rs3": -13}),
        ("addi", {"rd": -13, "rs1": -13, "imm": -21523360}),
        ("sw", {"rs2": 5, "rs1": -13, "imm": 9}),
        ("b", {"cond": -2, "hint": 1, "rs1": 1, "rs2": 2, "off": -797161}),
        ("br3", {"hint": -1, "rs1": 7, "offn": -3280, "offp": 3280}),
        ("lui", {"rd": 2, "imm": 581130733}),
        ("jal", {"rd": -12, "imm": -581130733}),
        ("tlut", {"rd": 1, "rs1": 2, "rs2": 3, "fn": I.TABLES["tmin"]}),
        ("cas", {"rd": 1, "rs1": 2, "rs2": 3, "rs3": 4}),
    ],
)
def test_encode_decode_round_trip(name, fields):
    op = I.OPS[name]
    d = I.decode(I.encode(op, **fields))
    assert d.op is op
    for k, v in fields.items():
        assert getattr(d, k) == v, k


def test_fields_that_do_not_fit_are_rejected():
    with pytest.raises(I.EncodingError):
        I.encode(I.OPS["addi"], rd=1, rs1=1, imm=21523361)


@pytest.mark.parametrize(
    ("name", "fields"),
    [
        ("abs", {"rd": 1, "rs1": 2, "rs2": 3}),  # abs uses no rs2
        ("add", {"rd": 1, "rs1": 2, "rs2": 3, "fn": 1}),
        ("sys", {"rd": 1}),
        ("b", {"cond": 4, "rs1": 1, "rs2": 2}),  # reserved condition
        ("amoadd", {"rd": 1, "rs1": 2, "rs2": 3, "rs3": -1}),  # ordering trit '-'
        ("tlut", {"rd": 1, "rs1": 2, "rs2": 3, "fn": 3**9}),
    ],
)
def test_must_be_zero_fields_make_an_instruction_illegal(name, fields):
    word = I.encode(I.OPS[name], **fields)
    with pytest.raises(I.IllegalInstruction):
        I.decode(word)


def test_csr_numbers_encode_their_access_rule():
    assert I.csr_ring(I.CSRS["mtvec"]) == 1
    assert I.csr_ring(I.CSRS["stvec"]) == 0
    assert I.csr_ring(I.CSRS["cycle"]) == -1
    assert I.csr_writable(I.CSRS["fcsr"])
    assert not I.csr_writable(I.CSRS["time"])


def test_disassembly_names_branch_conditions_and_hints():
    word = I.encode(I.OPS["b"], cond=I.CONDS["ge"], hint=1, rs1=1, rs2=2, off=2)
    assert disassemble(word, 30) == "bge.likely a0, a1, 36"
