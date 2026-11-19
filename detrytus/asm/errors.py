"""The assembler's error type, shared by parsing and both passes."""


class AsmError(Exception):
    """A problem in the source, located by file and line where known."""

    def __init__(
        self,
        message: str,
        line: int | None = None,
        source: str = "",
        file: str | None = None,
    ) -> None:
        if line is None:
            where = f"{file}: " if file else ""
        else:
            where = f"{file}:{line}: " if file else f"line {line}: "
        context = f"\n    {source.strip()}" if source else ""
        super().__init__(f"{where}{message}{context}")
        self.message = message  # without the location and source line
        self.line = line
        self.file = file
