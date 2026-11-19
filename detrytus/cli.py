"""Command line: ``detrytus asm`` lists, ``detrytus run`` executes."""

import argparse
import sys

from detrytus import __version__
from detrytus import image as images
from detrytus.asm import AsmError, Image, assemble, assemble_file
from detrytus.cpu import Hart
from detrytus.debug import Debugger, DebugShell
from detrytus.disasm import disassemble
from detrytus.isa import Decoded
from detrytus.machine import Machine, StepLimit
from detrytus.trap import UnhandledTrap
from detrytus.trits import format_trits


def _load_one(path: str) -> Image:
    """An assembly source file, an image file, or - for source on stdin."""
    if path == "-":
        return assemble(sys.stdin.read(), "<stdin>")
    if images.is_image(path):
        return images.read(path)
    return assemble_file(path)


def _load(paths: list[str]) -> Image:
    return images.merge([_load_one(p) for p in paths])


def cmd_asm(args: argparse.Namespace) -> int:
    image = _load_one(args.file)
    if args.output:
        images.write(image, args.output)
        if not args.listing:
            return 0
    for e in image.listing:
        # A section that runs somewhere other than where it loads shows both.
        where = str(e.addr) if e.load in (None, e.addr) else f"{e.addr}@{e.load}"
        trits = "" if e.word is None else format_trits(e.word)
        print(f"{where:>22}  {trits:27}  {e.text:28}  {e.source.strip()}")
    return 0


def cmd_run(args: argparse.Namespace) -> int:
    image = _load(args.files)

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


def cmd_debug(args: argparse.Namespace) -> int:
    image = _load(args.files)

    def out(ch: str) -> None:
        sys.stdout.write(ch)
        sys.stdout.flush()

    shell = DebugShell(Debugger(image, ram_pages=args.ram_pages, out=out))
    for expr in args.break_at:
        shell.onecmd(f"break {expr}")
    if args.commands:
        with open(args.commands, encoding="utf-8") as f:
            for line in f:
                if shell.onecmd(shell.precmd(line.rstrip("\n"))):
                    return 0
    if args.batch:
        return 0
    shell.say(shell.intro)
    shell.show_current()
    shell.cmdloop(intro="")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="detrytus", description=__doc__)
    parser.add_argument("--version", action="version", version=__version__)
    sub = parser.add_subparsers(dest="command", required=True)

    p = sub.add_parser(
        "asm", help="assemble a file: print its listing or write an image"
    )
    p.add_argument("file", help="assembly source, or - for stdin")
    p.add_argument(
        "-o", "--output", metavar="FILE", help="write the image to FILE (.dti)"
    )
    p.add_argument(
        "--listing", action="store_true", help="print the listing even with -o"
    )
    p.set_defaults(func=cmd_asm)

    p = sub.add_parser("run", help="assemble and run a program until it powers off")
    p.add_argument(
        "files",
        nargs="+",
        metavar="FILE",
        help="assembly sources or .dti images, loaded together; - for stdin",
    )
    p.add_argument("--max-steps", type=int, default=10_000_000)
    p.add_argument("--ram-pages", type=int, default=729)
    p.add_argument("--trace", action="store_true", help="print each instruction")
    p.add_argument("--stats", action="store_true", help="print the step count")
    p.set_defaults(func=cmd_run)

    p = sub.add_parser("debug", help="assemble a program and debug it interactively")
    p.add_argument(
        "files",
        nargs="+",
        metavar="FILE",
        help="assembly sources or .dti images, loaded together",
    )
    p.add_argument("--ram-pages", type=int, default=729)
    p.add_argument(
        "-b",
        "--break",
        dest="break_at",
        action="append",
        default=[],
        metavar="ADDR",
        help="set a breakpoint before starting (repeatable)",
    )
    p.add_argument(
        "-x", "--commands", metavar="FILE", help="run debugger commands from FILE first"
    )
    p.add_argument("--batch", action="store_true", help="exit after the -x commands")
    p.set_defaults(func=cmd_debug)

    args = parser.parse_args(argv)
    try:
        status: int = args.func(args)
        return status
    except (AsmError, images.ImageError) as e:
        print(f"detrytus: {e}", file=sys.stderr)
        return 65
    except OSError as e:
        print(f"detrytus: {e}", file=sys.stderr)
        return 66


if __name__ == "__main__":
    sys.exit(main())
