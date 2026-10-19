"""The command line, and the shape of a run.

A run is four steps: describe the boundary, work out how it is laid out, hand
that to each backend, and write what comes back. Only the first step talks to
the engine, and only the last writes anything.
"""

from __future__ import annotations

import argparse
import os
import shutil
import sys
import traceback
from pathlib import Path

from . import registry
from .ctypes_layout import build_ctypes
from .cparse import parse_header
from .ir import Abi, Layout, Target, UnknownType
from .source import (
    SourceError,
    build_wasm,
    engine_root,
    find_header,
    find_library,
    find_wasm,
    generate_header,
)

# WebAssembly is the one target where a pointer and a `size_t` are half the
# width they are everywhere else. Everything else the engine is built for is
# 64-bit on the desktop.
NATIVE = Target("native", pointer_size=8, size_t_size=8)
WASM32 = Target("wasm32", pointer_size=4, size_t_size=4)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="generate.py",
        description="Generate bindings for the Louder engine in every language the "
        "generator knows.",
    )
    parser.add_argument(
        "--lang",
        action="append",
        dest="languages",
        choices=registry.names(),
        metavar="LANGUAGE",
        help="Only write this language. May be given more than once.",
    )
    parser.add_argument(
        "--out",
        type=Path,
        default=None,
        help="Where to write the bindings. Defaults to `out` beside this script.",
    )
    parser.add_argument(
        "--engine",
        type=Path,
        default=None,
        help="The engine workspace. Found by walking up from this file if not given.",
    )
    parser.add_argument(
        "--header",
        type=Path,
        default=None,
        help="Use this C header instead of running cbindgen.",
    )
    parser.add_argument(
        "--wasm",
        type=Path,
        default=None,
        help="Use this WebAssembly module instead of building one.",
    )
    parser.add_argument(
        "--no-wasm-build",
        action="store_true",
        help="Do not build the engine for WebAssembly. The JavaScript bindings "
        "will be written without the module's own signatures.",
    )
    parser.add_argument(
        "--library",
        type=Path,
        default=None,
        help="The shared library to name in the bindings that load one.",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="List the languages and exit.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Report which languages would change, without writing anything.",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)

    if args.list:
        return _list()

    here = Path(__file__).resolve().parent.parent
    engine = args.engine.resolve() if args.engine else engine_root(here)
    out = (args.out or (here / "out")).resolve()

    languages = tuple(args.languages) if args.languages else tuple(registry.names())
    wanted = [
        name
        for name in registry.names()
        if name in languages and name in registry.REGISTRY
    ]
    for name, reason in registry.BROKEN.items():
        if name in languages:
            print(f"  {name} is unavailable: {reason}", file=sys.stderr)

    try:
        abi, wasm_path = _load(engine, out, args, wanted)
    except SourceError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2

    try:
        _report(abi, "the C header")
    except UnknownType as error:
        print(f"error: {error}", file=sys.stderr)
        return 2

    native_layout = Layout(abi, NATIVE)
    problems = native_layout.check_against_ctypes()
    if problems:
        print("error: the layout of the header does not agree with ctypes:", file=sys.stderr)
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        return 2

    if args.check:
        return _check(out, abi, native_layout, engine, wasm_path, wanted)

    written = 0
    options = _options(engine, out, args, None)
    for name in wanted:
        entry = registry.REGISTRY[name]
        # A backend that talks to a built module gets the layout for the machine
        # that module was built for, which is the one place pointers are not
        # eight bytes wide.
        layout = Layout(abi, WASM32) if entry.factory.needs_wasm and wasm_path else native_layout
        try:
            backend = registry.build(name, abi, layout, options)
            result = backend.generate()
        except NotImplementedError:
            print(f"  {entry.label:<14} skipped: the backend does not implement it yet")
            continue
        except Exception as error:  # noqa: BLE001 - one language must not stop the rest
            print(f"  {entry.label:<14} failed: {type(error).__name__}: {error}", file=sys.stderr)
            if os.environ.get("LOUDER_BINDINGS_TRACE"):
                traceback.print_exc()
            continue

        target = out / name
        for relative, text in result.files.items():
            path = target / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8", newline="\n")

        for relative, data in result.binaries.items():
            path = target / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)

        # A binding that loads a shared library is only worth having next to
        # something to load, so the engine is copied in beside it, under the
        # name its own platform expects.
        if entry.factory.needs_library and options.library and options.library.is_file():
            directory = _library_spot(result.files)
            if directory:
                destination = (target / directory).resolve()
                destination.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(options.library, destination / options.library.name)

        written += len(result.files)
        note = "" if not entry.factory.needs_wasm else " (wasm32)"
        print(f"  {entry.label:<14} {len(result.files)} files{note}")

    _write_index(out, abi, wanted)
    print(f"\n{written} files written under {out}")
    return 0


def _options(engine: Path, out: Path, args, wasm_path: Path | None):
    from .backends.base import Options

    library = args.library.resolve() if args.library else find_library(engine)
    return Options(
        out=out,
        engine=engine,
        library=library,
        wasm=wasm_path,
        languages=tuple(args.languages or ()),
    )


def _load(engine: Path, out: Path, args, wanted: list[str]) -> tuple[Abi, Path | None]:
    """Read the boundary, and the module if a language needs one."""
    if args.header:
        header = args.header.read_text(encoding="utf-8")
        header_path = args.header
    else:
        header, header_path = generate_header(engine, out / "abi")

    abi = parse_header(header)
    abi_dir = out / "abi"
    abi_dir.mkdir(parents=True, exist_ok=True)
    if header_path is not None and header_path.parent != abi_dir:
        shutil.copyfile(header_path, abi_dir / header_path.name)

    wasm_path: Path | None = None
    needs_wasm = any(registry.REGISTRY[name].factory.needs_wasm for name in wanted)
    if needs_wasm:
        if args.wasm:
            wasm_path = args.wasm.resolve()
        elif not args.no_wasm_build:
            try:
                wasm_path = build_wasm(engine)
            except SourceError:
                wasm_path = find_wasm(engine)
        else:
            wasm_path = find_wasm(engine)

        if wasm_path is not None:
            from .wasm import load as load_wasm

            # Reading the module resolves every signature against it, so a
            # binding can never disagree with what the engine actually exports.
            module = load_wasm(wasm_path)
            if not module.has_memory():
                raise SourceError(f"{wasm_path} does not export its memory, which the bindings need")
            abi.funcs = module.resolve(abi)
            shutil.copyfile(wasm_path, abi_dir / wasm_path.name)

    return abi, wasm_path


def _library_spot(files: dict[str, str]) -> str | None:
    """Where a backend wants the engine's library, given the files it wrote.

    A backend that keeps its code in a package directory has that directory as
    the one place a loader will certainly look, so that is where the library
    goes. Everything else loads by name from the platform's own search path and
    does not need a copy.
    """
    for relative in files:
        parts = relative.split("/")
        if parts[0] == "louder" and len(parts) > 1:
            return "/".join(parts[:-1])
    return None


def _report(abi: Abi, what: str) -> None:
    shapes = {
        "structs": len(abi.structs),
        "unions": len(abi.unions),
        "enums": len(abi.enums),
        "opaques": len(abi.opaques),
        "aliases": len(abi.aliases),
        "constants": len(abi.consts),
        "functions": len(abi.funcs),
    }
    summary = ", ".join(f"{count} {name}" for name, count in shapes.items())
    documented = sum(1 for f in abi.funcs if f.doc)
    print(f"read {what}: {summary}")
    print(f"  {documented} of {shapes['functions']} entry points carry the engine's own documentation")


def _list() -> int:
    grouped = registry.tiers()
    for tier in ("deep", "solid", "broad", "long"):
        labels = grouped.get(tier)
        if not labels:
            continue
        print(f"{tier}:")
        for label in labels:
            print(f"  {label}")
    print()
    print(f"{len(registry.names())} languages")
    return 0


def _check(out: Path, abi, layout, engine: Path, wasm_path, wanted) -> int:
    changed = 0
    for name in wanted:
        entry = registry.REGISTRY[name]
        try:
            backend = registry.build(name, abi, layout, _options(engine, out, argparse.Namespace(languages=[], library=None), wasm_path))
            result = backend.generate()
        except Exception as error:  # noqa: BLE001
            print(f"  {entry.label:<14} failed: {error}")
            changed += 1
            continue
        differing = []
        for relative, text in result.files.items():
            path = out / name / relative
            if not path.is_file() or path.read_text(encoding="utf-8") != text:
                differing.append(relative)
        if differing:
            changed += 1
            print(f"  {entry.label:<14} would change: {', '.join(differing)}")
        else:
            print(f"  {entry.label:<14} unchanged")
    print(f"\n{changed} of {len(wanted)} would change")
    return 0


def _write_index(out: Path, abi: Abi, written: list[str]) -> None:
    from .index import render_index

    (out / "README.md").write_text(render_index(abi, written), encoding="utf-8", newline="\n")
