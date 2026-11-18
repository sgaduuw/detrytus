import pytest

from detrytus import isa
from detrytus.disasm import disassemble


def test_zero_word_is_illegal():
    with pytest.raises(isa.IllegalInstruction):
        isa.decode(0)


def test_opcodes_are_unique_and_in_range():
    assert len({op.opcode for op in isa.OPS.values()}) == len(isa.OPS)
    assert all(-121 <= op.opcode <= 121 and op.opcode for op in isa.OPS.values())


def test_register_fields_share_positions_across_formats():
    # Section 6: rs1 sits at the same trits in every format that has it.
    rs1 = {isa.FORMAT_FIELDS[f]["rs1"] for f in "RISBT"}
    assert rs1 == {isa.RS1}


def test_formats_fill_exactly_27_trits():
    for fmt, fields in isa.FORMAT_FIELDS.items():
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
        ("tlut", {"rd": 1, "rs1": 2, "rs2": 3, "fn": isa.TABLES["tmin"]}),
        ("cas", {"rd": 1, "rs1": 2, "rs2": 3, "rs3": 4}),
    ],
)
def test_encode_decode_round_trip(name, fields):
    op = isa.OPS[name]
    d = isa.decode(isa.encode(op, **fields))
    assert d.op is op
    for k, v in fields.items():
        assert getattr(d, k) == v, k


def test_fields_that_do_not_fit_are_rejected():
    with pytest.raises(isa.EncodingError):
        isa.encode(isa.OPS["addi"], rd=1, rs1=1, imm=21523361)


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
    word = isa.encode(isa.OPS[name], **fields)
    with pytest.raises(isa.IllegalInstruction):
        isa.decode(word)


def test_csr_numbers_encode_their_access_rule():
    assert isa.csr_ring(isa.CSRS["mtvec"]) == 1
    assert isa.csr_ring(isa.CSRS["stvec"]) == 0
    assert isa.csr_ring(isa.CSRS["cycle"]) == -1
    assert isa.csr_writable(isa.CSRS["fcsr"])
    assert not isa.csr_writable(isa.CSRS["time"])


def test_disassembly_names_branch_conditions_and_hints():
    word = isa.encode(isa.OPS["b"], cond=isa.CONDS["ge"], hint=1, rs1=1, rs2=2, off=2)
    assert disassemble(word, 30) == "bge.likely a0, a1, 36"
