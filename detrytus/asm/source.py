"""Source text to statements: comments, labels, mnemonics, operands."""

import re
from dataclasses import dataclass

_LABEL = re.compile(r"\s*([A-Za-z_.][A-Za-z0-9_.$]*)\s*:")


@dataclass
class Statement:
    line: int
    source: str
    labels: list[str]
    mnemonic: str  # '' for a label-only line
    operands: list[str]
    # Filled in by layout:
    addr: int = 0
    size: int = 0  # words, or trytes for tryte data
    kind: str = "word"  # 'word', 'tryte', or 'none'


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


def parse(source: str) -> list[Statement]:
    stmts = []
    for n, raw in enumerate(source.splitlines(), start=1):
        text = strip_comment(raw)
        labels = []
        while m := _LABEL.match(text):
            labels.append(m.group(1))
            text = text[m.end() :]
        text = text.strip()
        if not text:
            if labels:
                stmts.append(Statement(n, raw, labels, "", []))
            continue
        mnemonic, *rest = text.split(None, 1)
        operands = split_operands(rest[0]) if rest else []
        stmts.append(Statement(n, raw, labels, mnemonic.lower(), operands))
    return stmts
