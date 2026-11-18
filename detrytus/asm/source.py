"""Source text to statements: comments, labels, includes, operands.

Two transformations happen here, before layout sees anything: .include
splices another file's statements in place, and numeric local labels
(1:, referenced as 1b or 1f) are renamed to unique names.
"""

import re
from dataclasses import dataclass
from pathlib import Path

from detrytus.asm.errors import AsmError

_LABEL = re.compile(r"\s*(\d+|[A-Za-z_.][A-Za-z0-9_.$]*)\s*:")
# 1b or 1f, not part of a longer name or number such as 0x1b.
_LOCAL_REF = re.compile(r"(?<![\w.$'])(\d+)([bf])(?![\w.$])")
_TEXT_DIRECTIVES = (".string", ".ascii", ".include")
MAX_INCLUDE_DEPTH = 32


@dataclass
class Statement:
    line: int
    source: str
    labels: list[str]
    mnemonic: str  # '' for a label-only line
    operands: list[str]
    file: str | None = None
    # Filled in by layout:
    addr: int = 0
    size: int = 0  # words, or trytes for tryte data
    kind: str = "word"  # 'word', 'tryte', or 'none'
    shift: int = 0  # load address minus run address, from its section


def strip_comment(line: str) -> str:
    """Drop a '#' or ';' comment, ignoring those inside quotes."""
    quote = None
    for i, c in enumerate(line):
        if quote:
            if c == quote and line[i - 1] != "\\":
                quote = None
        elif c in "\"'":
            quote = c
        elif c in "#;":
            return line[:i]
    return line


def split_operands(text: str) -> list[str]:
    """Split on commas outside parentheses and quotes."""
    out: list[str] = []
    cur: list[str] = []
    depth = 0
    quote = None
    for i, c in enumerate(text):
        if quote:
            if c == quote and text[i - 1] != "\\":
                quote = None
        elif c in "\"'":
            quote = c
        elif c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
        elif c == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
            continue
        cur.append(c)
    if "".join(cur).strip():
        out.append("".join(cur).strip())
    return out


def _statement(n: int, raw: str, file: str | None) -> Statement | None:
    text = strip_comment(raw)
    labels = []
    while m := _LABEL.match(text):
        labels.append(m.group(1))
        text = text[m.end() :]
    text = text.strip()
    if not text:
        return Statement(n, raw, labels, "", [], file) if labels else None
    mnemonic, *rest = text.split(None, 1)
    operands = split_operands(rest[0]) if rest else []
    return Statement(n, raw, labels, mnemonic.lower(), operands, file)


def _include_path(s: Statement, base: Path) -> Path:
    if len(s.operands) != 1 or not re.fullmatch(r'"[^"]+"', s.operands[0]):
        raise AsmError('.include takes one "path"', s.line, s.source, s.file)
    return (base / s.operands[0][1:-1]).resolve()


def _parse_into(
    out: list[Statement], text: str, file: str | None, base: Path, stack: list[Path]
) -> None:
    for n, raw in enumerate(text.splitlines(), start=1):
        s = _statement(n, raw, file)
        if s is None:
            continue
        if s.mnemonic != ".include":
            out.append(s)
            continue
        if s.labels:  # labels on an include line name what follows it
            out.append(Statement(n, raw, s.labels, "", [], file))
        path = _include_path(s, base)
        if path in stack:
            raise AsmError(f"{path.name} includes itself", s.line, s.source, file)
        if len(stack) >= MAX_INCLUDE_DEPTH:
            raise AsmError("includes nested too deeply", s.line, s.source, file)
        try:
            included = path.read_text(encoding="utf-8")
        except OSError as e:
            raise AsmError(
                f"cannot include {path}: {e.strerror}", s.line, s.source, file
            ) from None
        _parse_into(out, included, str(path), path.parent, [*stack, path])


LocalDefs = dict[str, list[tuple[int, str]]]


def _number_local_labels(stmts: list[Statement]) -> LocalDefs:
    """Rename each numeric label uniquely; return where each one is defined."""
    defs: LocalDefs = {}
    for i, s in enumerate(stmts):
        for j, label in enumerate(s.labels):
            if label.isdigit():
                occurrences = defs.setdefault(label, [])
                unique = f".L{label}.{len(occurrences)}"
                occurrences.append((i, unique))
                s.labels[j] = unique
    return defs


def _resolve_local(defs: LocalDefs, i: int, s: Statement, m: re.Match[str]) -> str:
    """1b is the nearest definition of 1 at or before statement i (a label
    on the same line counts), 1f the nearest one after it."""
    number, direction = m.group(1), m.group(2)
    candidates = defs.get(number, [])
    if direction == "b":
        found = [u for at, u in candidates if at <= i]
        if found:
            return found[-1]
    else:
        found = [u for at, u in candidates if at > i]
        if found:
            return found[0]
    where = "before" if direction == "b" else "after"
    raise AsmError(
        f"no local label {number} {where} this line", s.line, s.source, s.file
    )


def _rename_local_labels(stmts: list[Statement]) -> None:
    """Give each numeric label a unique name and point 1b/1f references at it."""
    defs = _number_local_labels(stmts)
    for i, s in enumerate(stmts):
        if s.mnemonic in _TEXT_DIRECTIVES:
            continue

        def replace(m: re.Match[str], i: int = i, s: Statement = s) -> str:
            return _resolve_local(defs, i, s, m)

        s.operands = [_LOCAL_REF.sub(replace, op) for op in s.operands]


def parse(
    source: str, file: str | None = None, base: Path | None = None
) -> list[Statement]:
    """Statements of source. file names it in errors; includes are found
    relative to base, which defaults to the file's directory or the
    current directory."""
    if base is None:
        base = Path(file).resolve().parent if file else Path.cwd()
    stack = [Path(file).resolve()] if file else []
    stmts: list[Statement] = []
    _parse_into(stmts, source, file, base, stack)
    _rename_local_labels(stmts)
    return stmts
