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


# --- sections --------------------------------------------------------------------


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


# --- run addresses -----------------------------------------------------------------


RUN_ELSEWHERE = """
boot:   li    t0, entry          # another section's label: its run address
        jalr  zero, 0(t0)
        .section moved, 300, -3000
entry:  la    a0, data           # pc-relative within the section
        j     entry
data:   .word 7
"""


def test_a_section_runs_at_its_run_address_and_loads_at_its_load_address():
    image = assemble(RUN_ELSEWHERE)
    assert image.labels["entry"] == -3000
    assert image.labels["data"] == -3000 + 9  # la (2 words) and j (1)
    # The trytes sit at the load address; nothing is placed at the run address.
    assert {300 - 1, 300, 301} <= image.trytes.keys()
    assert not image.trytes.keys() & set(range(-3002, -2990))
    texts_by_addr = {e.addr: e.text for e in image.listing if e.word is not None}
    assert texts_by_addr[-3000 + 6] == "jal zero, -3000"  # branch encoded for -3000
    assert {e.load for e in image.listing if e.addr == -3000} == {300}


def test_li_of_a_label_gives_its_run_address():
    image = assemble(RUN_ELSEWHERE)
    # A forward reference takes li's long form; -3000 fits its low 8 trits.
    assert texts(image)[:2] == ["lui t0, 0", "addi t0, t0, -3000"]


def test_the_paging_example_runs_with_virtual_labels():
    image = assemble_file(EXAMPLES / "paging.s")
    assert image.labels["user"] == 19683  # virtual page 1
    m = Machine()
    m.load(image)
    assert m.run(10_000).exit_code == 0
