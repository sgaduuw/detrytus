"""The assembler. Syntax is described in docs/assembler.md.

source.py turns text into statements (following .include and renaming
numeric local labels), operands.py turns operand text
into instruction fields, pseudo.py expands pseudo-instructions, and
core.py runs the two passes.
"""

from detrytus.asm.core import Assembler, Image, Listing, assemble, assemble_file
from detrytus.asm.errors import AsmError

__all__ = ["AsmError", "Assembler", "Image", "Listing", "assemble", "assemble_file"]
