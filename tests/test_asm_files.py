"""Includes, numeric local labels and sections."""

from pathlib import Path

import pytest

from detrytus.asm import AsmError, assemble, assemble_file
from detrytus.machine import Machine

EXAMPLES = Path(__file__).parent.parent / "examples"


def texts(image):
    return [e.text for e in image.listing if e.word is not None]


# --- .include ------------------------------------------------------------------


def test_include_splices_a_file_relative_to_the_includer(tmp_path):
    (tmp_path / "lib").mkdir()
    (tmp_path / "lib" / "a.s").write_text('.include "b.s"\nfrom_a: nop\n')
    (tmp_path / "lib" / "b.s").write_text("from_b: nop\n")
    main = tmp_path / "main.s"
    main.write_text('start: nop\n.include "lib/a.s"\n')
    image = assemble_file(main)
    # b.s is found next to a.s, not next to main.s or the cwd.
    assert image.labels == {"start": 0, "from_b": 3, "from_a": 6}


def test_labels_on_an_include_line_name_what_it_brings_in(tmp_path):
    (tmp_path / "x.s").write_text("nop\n")
    main = tmp_path / "main.s"
    main.write_text('nop\nhere: .include "x.s"\n')
    assert assemble_file(main).labels["here"] == 3


def test_an_include_cycle_is_an_error(tmp_path):
    (tmp_path / "a.s").write_text('.include "b.s"\n')
    (tmp_path / "b.s").write_text('.include "a.s"\n')
    with pytest.raises(AsmError, match="includes itself"):
        assemble_file(tmp_path / "a.s")


def test_a_missing_include_names_the_file_and_line(tmp_path):
    main = tmp_path / "main.s"
    main.write_text('nop\n.include "nope.s"\n')
    with pytest.raises(AsmError, match=r"main\.s:2: cannot include"):
        assemble_file(main)


def test_errors_inside_an_included_file_point_into_it(tmp_path):
    (tmp_path / "bad.s").write_text("nop\nfrob a0\n")
    main = tmp_path / "main.s"
    main.write_text('.include "bad.s"\n')
    with pytest.raises(AsmError, match=r"bad\.s:2: unknown instruction") as e:
        assemble_file(main)
    assert e.value.line == 2


def test_errors_in_a_string_source_still_give_the_line():
    with pytest.raises(AsmError, match=r"^line 2: "):
        assemble("nop\nfrob")


# --- numeric local labels --------------------------------------------------------


def test_backward_and_forward_local_labels():
    image = assemble(
        """
1:      addi a0, a0, 1      # 0
        blt  a0, a1, 1b     # 3: back to 0, not to a later 1
        j    1f             # 6: on to the next 1, at 9, not the one at 12
1:      nop                 # 9
1:      nop                 # 12
    """
    )
    assert texts(image) == [
        "addi a0, a0, 1",
        "blt a0, a1, 0",
        "jal zero, 9",
        "addi zero, zero, 0",
        "addi zero, zero, 0",
    ]


def test_a_label_on_the_same_line_counts_as_before():
    assert texts(assemble("1: j 1b")) == ["jal zero, 0"]


def test_a_forward_reference_skips_its_own_line():
    # 1f on a line that defines 1 means the next 1, not this one.
    assert texts(assemble("1: j 1f\n1: nop"))[0] == "jal zero, 3"


def test_each_reference_picks_the_nearest_definition():
    image = assemble("1: nop\nj 1b\n1: nop\nj 1b\nj 1f\n1: nop")
    assert texts(image)[1] == "jal zero, 0"  # nearest earlier 1 is at 0
    assert texts(image)[3] == "jal zero, 6"  # now the one at 6
    assert texts(image)[4] == "jal zero, 15"


@pytest.mark.parametrize(
    ("source", "where"), [("j 3f", "after"), ("j 3b\n3: nop", "before")]
)
def test_a_missing_local_label_is_an_error(source, where):
    with pytest.raises(AsmError, match=f"no local label 3 {where} this line"):
        assemble(source)


def test_hex_literals_and_strings_are_not_local_references():
    image = assemble('li a0, 0x1b\n.org 30\ns: .string "1b"')
    assert texts(image) == ["addi a0, zero, 27"]
    assert [image.trytes[29], image.trytes[30]] == [ord("1"), ord("b")]


# --- sections --------------------------------------------------------------------


def test_sections_place_interleaved_code_and_data_apart():
    image = assemble(
        """
a:      nop
        .section data, 300
x:      .word 1
        .section text
b:      nop
        .section data
y:      .word 2
    """
    )
    assert image.labels == {"a": 0, "x": 300, "b": 3, "y": 303}


def test_a_new_section_needs_an_address():
    with pytest.raises(AsmError, match="needs an address the first time"):
        assemble(".section data\nnop")


def test_a_section_is_placed_once():
    with pytest.raises(AsmError, match="already placed"):
        assemble(".section data, 300\n.section text\n.section data, 600")


def test_a_section_starts_word_aligned():
    with pytest.raises(AsmError, match="word-aligned"):
        assemble(".section data, 301")


def test_a_label_before_a_switch_names_the_old_section_end():
    image = assemble("nop\nend_of_text:\n.section data, 300\n.word 5")
    assert image.labels["end_of_text"] == 2  # the next free tryte of text


def test_overlapping_sections_are_caught():
    with pytest.raises(AsmError, match="assembled twice"):
        assemble("nop\nnop\n.section data, 3\n.word 1")


def test_sections_example_runs():
    image = assemble_file(EXAMPLES / "sections.s")
    m = Machine()
    m.load(image)
    assert m.run(10_000).exit_code == 0
    assert m.console.text.splitlines() == [
        "code in the boot page, strings in RAM page 2",
        "helpers came from lib/console.s",
    ]
    # Like .org, a section address names where the next word is centred,
    # so tryte data begins one tryte below it.
    assert image.labels["greeting"] == 2 * 19683 - 1


def test_listing_entries_name_the_file_they_came_from(tmp_path):
    (tmp_path / "data.s").write_text('.org 30\nmsg: .string "hi"\n')
    main = tmp_path / "main.s"
    main.write_text('nop\n.include "data.s"\n')
    by_file = {e.file: e for e in assemble_file(main).listing}
    included = by_file[str((tmp_path / "data.s").resolve())]
    assert included.word is None  # the string
    assert included.text == ""  # no disassembly, and not a file name either
    assert by_file[str(main)].text == "addi zero, zero, 0"
