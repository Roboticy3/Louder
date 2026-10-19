"""Finding the engine, and asking it to describe its own boundary.

`cbindgen` already knows how to read the Rust and write a header that describes
the boundary exactly. Running it here rather than parsing the Rust keeps one
description of the ABI in the project instead of two, and means the bindings
cannot say anything the C header does not.

The same is true of the WebAssembly module: it is built by cargo, and its own
type section is what the JavaScript binding is written against.
"""

from __future__ import annotations

import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path


class SourceError(Exception):
    """Raised when the boundary cannot be read from the engine."""


@dataclass
class Sources:
    """Where the description of the boundary came from."""

    header: str
    header_path: Path | None
    wasm_path: Path | None
    wasm_built: bool


def engine_root(start: Path) -> Path:
    """The `engine` directory, found by walking up from this file."""
    here = start.resolve()
    for candidate in [here, *here.parents]:
        if (candidate / "louder_rust" / "Cargo.toml").is_file():
            return candidate
    raise SourceError("could not find the engine workspace from " + str(start))


def generate_header(engine: Path, out: Path, *, lang: str = "C", docs: bool = True) -> tuple[str, Path]:
    """Run `cbindgen` over the engine crate and return the header it wrote.

    The engine's own config leaves documentation off, because the C++ tests
    compile against the header and do not read it. The bindings are the other
    way round: they are often the first place someone meets an entry point, so
    the header is generated with the prose turned on. Only that one setting is
    overridden, so there is still a single config describing the boundary.
    """
    if shutil.which("cbindgen") is None:
        raise SourceError(
            "cbindgen is not on PATH. Install it with `cargo install cbindgen`, "
            "or pass --header with a header generated earlier."
        )

    crate = engine / "louder_rust"
    config = crate / "cbindgen.toml"
    if docs and config.is_file():
        active = out / "cbindgen.bindings.toml"
        active.parent.mkdir(parents=True, exist_ok=True)
        active.write_text(_with_documentation(config.read_text(encoding="utf-8")), encoding="utf-8")
    else:
        active = config

    target = out / f"louder_rust.{lang.lower()}.h"
    target.parent.mkdir(parents=True, exist_ok=True)

    result = subprocess.run(
        [
            "cbindgen",
            "--config",
            str(active),
            "--crate",
            "louder_rust",
            "--lang",
            lang,
            "--output",
            str(target),
        ],
        cwd=crate,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SourceError(f"cbindgen failed with exit code {result.returncode}:\n{result.stderr}")

    return target.read_text(encoding="utf-8"), target


def _with_documentation(text: str) -> str:
    import re

    if re.search(r"^\s*documentation\s*=", text, re.MULTILINE):
        return re.sub(r"^(\s*documentation\s*=\s*)\w+", r"\1true", text, count=1, flags=re.MULTILINE)
    return text + "\ndocumentation = true\n"


def build_wasm(engine: Path, *, profile: str = "release", target: str = "wasm32-unknown-unknown") -> Path:
    """Build the engine for WebAssembly and return the module.

    The crate is a plain `cdylib`, so the exports land under their C names and
    the module can be called from JavaScript with no glue step in between.
    """
    if shutil.which("cargo") is None:
        raise SourceError("cargo is not on PATH, so the WebAssembly module cannot be built")

    result = subprocess.run(
        ["cargo", "build", "-p", "louder_rust", "--target", target]
        + (["--release"] if profile == "release" else []),
        cwd=engine,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SourceError(f"the WebAssembly build failed with exit code {result.returncode}:\n{result.stderr}")

    module = engine / "target" / target / profile / "louder_rust.wasm"
    if not module.is_file():
        raise SourceError(f"cargo reported success but there is no module at {module}")
    return module


def find_wasm(engine: Path, target: str = "wasm32-unknown-unknown") -> Path | None:
    for profile in ("release", "debug"):
        candidate = engine / "target" / target / profile / "louder_rust.wasm"
        if candidate.is_file():
            return candidate
    return None


def find_header(engine: Path) -> Path | None:
    """A C header cbindgen has already written, if there is one to reuse."""
    for candidate in (
        engine / "artifacts" / "louder_rust.h",
        engine / "louder_rust" / "louder_rust.h",
        engine / "target" / "debug" / "louder_rust.h",
    ):
        if candidate.is_file():
            return candidate
    return None


def find_library(engine: Path) -> Path | None:
    """The engine's shared library, for the bindings that load one."""
    for profile in ("debug", "release"):
        for name in ("louder_rust.dll", "liblouder_rust.so", "liblouder_rust.dylib", "louder_rust.dll", "louder_rust.so", "louder_rust.dylib"):
            candidate = engine / "target" / profile / name
            if candidate.is_file():
                return candidate
    return None
