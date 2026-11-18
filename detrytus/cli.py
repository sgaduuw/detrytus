"""Command line: ``detrytus asm`` lists, ``detrytus run`` executes."""

import argparse
import sys

from detrytus import __version__
from detrytus.asm import AsmError, assemble
from detrytus.cpu import Hart
from detrytus.disasm import disassemble
from detrytus.isa import Decoded
from detrytus.machine import Machine, StepLimit
from detrytus.trap import UnhandledTrap
from detrytus.trits import format_trits


def _read(path: str) -> str:
    if path == "-":
        return sys.stdin.read()
    with open(path, encoding="utf-8") as f:
        return f.read()


def cmd_asm(args: argparse.Namespace) -> int:
    image = assemble(_read(args.file))
    for e in image.listing:
        if e.word is None:
            print(f"{e.addr:>14}  {'':27}  {'':28}  {e.source.strip()}")
        else:
            print(
                f"{e.addr:>14}  {format_trits(e.word)}  {e.text:28}  {e.source.strip()}"
            )
    return 0


def cmd_run(args: argparse.Namespace) -> int:
    image = assemble(_read(args.file))

    def out(ch: str) -> None:
        sys.stdout.write(ch)
        sys.stdout.flush()

    machine = Machine(ram_pages=args.ram_pages, out=out)
    machine.load(image)
    if args.trace:

        def trace(hart: Hart, d: Decoded) -> None:
            print(
                f"[{hart.pc:>12} r{hart.ring:+d}] {disassemble(d.word, hart.pc)}",
                file=sys.stderr,
            )

        machine.hart.trace = trace
    try:
        result = machine.run(args.max_steps)
    except UnhandledTrap as e:
        print(f"\ndetrytus: {e}", file=sys.stderr)
        return 70
    except StepLimit as e:
        print(f"\ndetrytus: {e}", file=sys.stderr)
        return 124
    except NotImplementedError as e:
        print(f"\ndetrytus: not implemented yet: {e}", file=sys.stderr)
        return 69
    if args.stats:
        print(
            f"\ndetrytus: {result.steps} steps, exit {result.exit_code}",
            file=sys.stderr,
        )
    return result.exit_code


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="detrytus", description=__doc__)
    parser.add_argument("--version", action="version", version=__version__)
    sub = parser.add_subparsers(dest="command", required=True)

    p = sub.add_parser("asm", help="assemble a file and print its listing")
    p.add_argument("file", help="assembly source, or - for stdin")
    p.set_defaults(func=cmd_asm)

    p = sub.add_parser("run", help="assemble and run a program until it powers off")
    p.add_argument("file", help="assembly source, or - for stdin")
    p.add_argument("--max-steps", type=int, default=10_000_000)
    p.add_argument("--ram-pages", type=int, default=729)
    p.add_argument("--trace", action="store_true", help="print each instruction")
    p.add_argument("--stats", action="store_true", help="print the step count")
    p.set_defaults(func=cmd_run)

    args = parser.parse_args(argv)
    try:
        status: int = args.func(args)
        return status
    except AsmError as e:
        print(f"detrytus: {e}", file=sys.stderr)
        return 65
    except OSError as e:
        print(f"detrytus: {e}", file=sys.stderr)
        return 66


if __name__ == "__main__":
    sys.exit(main())
