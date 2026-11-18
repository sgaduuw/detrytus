"""The .dti image format and loading several images together."""

from pathlib import Path

import pytest

from detrytus import image as images
from detrytus.asm import assemble, assemble_file
from detrytus.cli import main
from detrytus.machine import Machine
from detrytus.trits import TRYTE_MAX

EXAMPLES = Path(__file__).parent.parent / "examples"


@pytest.mark.parametrize("trytes", [[], [0], [TRYTE_MAX, -TRYTE_MAX, 1, -1, 0, 4242]])
def test_pack_round_trips(trytes):
    data = images.pack(trytes)
    assert len(data) == -(-9 * len(trytes) // 5)  # five trits a byte
    assert all(b <= 242 for b in data)
    assert images.unpack(data, len(trytes)) == trytes


@pytest.mark.parametrize("name", ["hello.s", "bsearch.s", "paging.s", "sections.s"])
def test_examples_survive_a_round_trip(name):
    original = assemble_file(EXAMPLES / name)
    loaded = images.loads(images.dumps(original))
    assert loaded.trytes == original.trytes
    assert loaded.labels == original.labels
    assert loaded.symbols == original.symbols


def test_a_loaded_image_runs_like_the_source(tmp_path):
    path = tmp_path / "hello.dti"
    images.write(assemble_file(EXAMPLES / "hello.s"), path)
    m = Machine()
    m.load(images.read(path))
    assert m.run(10_000).exit_code == 0
    assert m.console.text == "Hello from a balanced-ternary machine!\n"


def good() -> bytes:
    return images.dumps(assemble("nop\nnop"))


@pytest.mark.parametrize(
    ("data", "message"),
    [
        (b"ELF!" + good()[4:], "not a detrytus image"),
        (good()[:-1], "truncated"),
        (good() + b"\x00", "trailing bytes"),
        (good()[:-1] + bytes([243]), "not five trits"),
    ],
)
def test_corrupt_images_are_rejected(data, message):
    with pytest.raises(images.ImageError, match=message):
        images.loads(data)


def test_merge_combines_disjoint_images():
    a = assemble("start: nop\n.equ PAGE, 19683")
    b = assemble(".org 300\nuser: nop\n.equ PAGE, 19683")
    merged = images.merge([a, b])
    assert merged.trytes.keys() == a.trytes.keys() | b.trytes.keys()
    assert merged.labels == {"start": 0, "user": 300}
    assert merged.symbols["PAGE"] == 19683  # same value in both: kept


def test_merge_rejects_overlap():
    with pytest.raises(images.ImageError, match="overlap at address"):
        images.merge([assemble("nop"), assemble("nop")])


def test_merge_drops_names_with_two_values():
    merged = images.merge([assemble("start: nop"), assemble(".org 300\nstart: nop")])
    assert "start" not in merged.labels
    assert "start" not in merged.symbols


FIRMWARE = """
        li    t0, 5 * 19683
        jalr  zero, 0(t0)      # into the separately built program
"""
PROGRAM = """
        .org  5 * 19683
        li    a0, 42
        li    t0, -39366
        sw    a0, 0(t0)
"""


def test_cli_builds_and_runs_separate_images(tmp_path):
    (tmp_path / "fw.s").write_text(FIRMWARE)
    (tmp_path / "prog.s").write_text(PROGRAM)
    assert (
        main(["asm", str(tmp_path / "prog.s"), "-o", str(tmp_path / "prog.dti")]) == 0
    )
    assert images.is_image(tmp_path / "prog.dti")
    # A source and an image, loaded together.
    assert main(["run", str(tmp_path / "fw.s"), str(tmp_path / "prog.dti")]) == 42


def test_cli_reports_overlapping_images(tmp_path, capsys):
    (tmp_path / "a.s").write_text("nop\n")
    assert main(["run", str(tmp_path / "a.s"), str(tmp_path / "a.s")]) == 65
    assert "overlap" in capsys.readouterr().err
