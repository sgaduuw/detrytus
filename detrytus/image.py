"""Assembled images on disk, and loading several of them together.

Layout of a .dti file, all integers little-endian:

    magic        4 bytes   b"DTI" and a format version byte (1)
    meta length  u32
    meta         UTF-8 JSON: {"labels": {name: addr}, "symbols": {name: value}}
    segments     u32 count, then for each:
        start    i64       address of the first tryte
        trytes   u32       how many trytes follow
        data     ceil(9 * trytes / 5) bytes

Data packs the trytes' trits, least significant first, five to a byte:
a byte is sum((trit_i + 1) * 3**i), at most 242 (docs/isa.md 11). The
last byte is padded with zero trits.
"""

import json
import struct
from pathlib import Path

from detrytus.asm import Image
from detrytus.trits import TRYTE_MAX, TRYTE_TRITS, from_trits, pow3, to_trits

MAGIC = b"DTI\x01"
SUFFIX = ".dti"
TRITS_PER_BYTE = 5
BYTE_MAX = pow3(TRITS_PER_BYTE) - 1  # 242


class ImageError(ValueError):
    pass


def _segments(trytes: dict[int, int]) -> list[tuple[int, list[int]]]:
    """Runs of consecutive addresses, in address order."""
    runs: list[tuple[int, list[int]]] = []
    for addr in sorted(trytes):
        if runs and runs[-1][0] + len(runs[-1][1]) == addr:
            runs[-1][1].append(trytes[addr])
        else:
            runs.append((addr, [trytes[addr]]))
    return runs


def pack(trytes: list[int]) -> bytes:
    trits = [t for tryte in trytes for t in to_trits(tryte, TRYTE_TRITS)]
    trits += [0] * (-len(trits) % TRITS_PER_BYTE)
    return bytes(
        sum((t + 1) * pow3(i) for i, t in enumerate(trits[k : k + TRITS_PER_BYTE]))
        for k in range(0, len(trits), TRITS_PER_BYTE)
    )


def unpack(data: bytes, count: int) -> list[int]:
    trits = []
    for byte in data:
        if byte > BYTE_MAX:
            raise ImageError(f"byte {byte} is not five trits")
        for _ in range(TRITS_PER_BYTE):
            trits.append(byte % 3 - 1)
            byte //= 3
    needed = count * TRYTE_TRITS
    if len(trits) < needed:
        raise ImageError("segment data is shorter than its tryte count")
    return [
        from_trits(trits[k : k + TRYTE_TRITS]) for k in range(0, needed, TRYTE_TRITS)
    ]


def dumps(image: Image) -> bytes:
    meta = json.dumps(
        {"labels": image.labels, "symbols": image.symbols}, sort_keys=True
    ).encode()
    out = [MAGIC, struct.pack("<I", len(meta)), meta]
    segments = _segments(image.trytes)
    out.append(struct.pack("<I", len(segments)))
    for start, trytes in segments:
        out += [struct.pack("<qI", start, len(trytes)), pack(trytes)]
    return b"".join(out)


class _Reader:
    def __init__(self, data: bytes) -> None:
        self.data = data
        self.pos = 0

    def take(self, n: int) -> bytes:
        if self.pos + n > len(self.data):
            raise ImageError("image is truncated")
        chunk = self.data[self.pos : self.pos + n]
        self.pos += n
        return chunk

    def unpack(self, fmt: str) -> tuple[int, ...]:
        values: tuple[int, ...] = struct.unpack(fmt, self.take(struct.calcsize(fmt)))
        return values


def loads(data: bytes) -> Image:
    r = _Reader(data)
    if r.take(len(MAGIC)) != MAGIC:
        raise ImageError("not a detrytus image (or an unknown version)")
    (meta_len,) = r.unpack("<I")
    try:
        meta = json.loads(r.take(meta_len))
    except ValueError as e:
        raise ImageError(f"bad image metadata: {e}") from None
    image = Image(labels=dict(meta["labels"]), symbols=dict(meta["symbols"]))
    (count,) = r.unpack("<I")
    for _ in range(count):
        start, n = r.unpack("<qI")
        values = unpack(r.take(-(-n * TRYTE_TRITS // TRITS_PER_BYTE)), n)
        image.trytes.update(zip(range(start, start + n), values, strict=True))
    if r.pos != len(data):
        raise ImageError("trailing bytes after the last segment")
    if any(abs(t) > TRYTE_MAX for t in image.trytes.values()):
        raise ImageError("a tryte is out of range")
    return image


def write(image: Image, path: str | Path) -> None:
    Path(path).write_bytes(dumps(image))


def read(path: str | Path) -> Image:
    return loads(Path(path).read_bytes())


def is_image(path: str | Path) -> bool:
    try:
        with open(path, "rb") as f:
            return f.read(len(MAGIC)) == MAGIC
    except OSError:
        return False


def merge(images: list[Image]) -> Image:
    """Several images as one, for loading a kernel with its programs.

    Overlapping trytes are an error: one image would silently corrupt
    another. A name defined with different values in two images (two
    programs that each have a 'start') is dropped from the merged tables,
    so the debugger refuses it rather than picking the wrong one.
    """
    merged = Image()
    ambiguous: set[str] = set()
    for image in images:
        clash = merged.trytes.keys() & image.trytes.keys()
        if clash:
            raise ImageError(f"images overlap at address {min(clash)}")
        merged.trytes.update(image.trytes)
        for table, incoming in (
            (merged.symbols, image.symbols),
            (merged.labels, image.labels),
        ):
            for name, value in incoming.items():
                if table.get(name, value) != value:
                    ambiguous.add(name)
                table[name] = value
        merged.listing += image.listing
    for name in ambiguous:
        merged.symbols.pop(name, None)
        merged.labels.pop(name, None)
    return merged
