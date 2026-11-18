"""The assembler. Syntax is described in docs/assembler.md.

source.py turns text into statements, operands.py turns operand text
into instruction fields, pseudo.py expands pseudo-instructions, and
core.py runs the two passes.
"""

from detrytus.asm.core import AsmError, Assembler, Image, Listing, assemble

__all__ = ["AsmError", "Assembler", "Image", "Listing", "assemble"]
