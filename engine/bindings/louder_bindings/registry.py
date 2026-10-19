"""Which backends exist, and in what order they are written out.

The order is the order of effort, so a run that is interrupted leaves the
languages that matter most already on disk. The tier is what the table in the
README is sorted by, and what tells a reader how much polish a given language
got.
"""

from __future__ import annotations

import importlib
from dataclasses import dataclass

from .backends.base import Backend, Options

# (module, class, label, tier). The tiers, in order:
#   deep   a raw layer and a fully idiomatic one on top of it
#   solid  complete bindings, compiled rather than reinterpreted
#   broad  complete bindings with a thin convenience layer
#   long   declarations only, for the tail of FFI-capable languages
BACKENDS: tuple[tuple[str, str, str, str], ...] = (
    ("python", "PythonBackend", "Python", "deep"),
    ("javascript", "JavaScriptBackend", "JavaScript", "deep"),
    ("typescript", "TypeScriptBackend", "TypeScript", "deep"),
    ("rust", "RustBackend", "Rust", "solid"),
    ("go", "GoBackend", "Go", "solid"),
    ("csharp", "CSharpBackend", "C#", "solid"),
    ("java", "JavaBackend", "Java", "solid"),
    ("c", "CBackend", "C", "broad"),
    ("cpp", "CppBackend", "C++", "broad"),
    ("zig", "ZigBackend", "Zig", "broad"),
    ("swift", "SwiftBackend", "Swift", "broad"),
    ("kotlin", "KotlinBackend", "Kotlin", "broad"),
    ("lua", "LuaBackend", "Lua", "broad"),
    ("ruby", "RubyBackend", "Ruby", "broad"),
    ("php", "PhpBackend", "PHP", "broad"),
    ("dlang", "DBackend", "D", "broad"),
    ("nim", "NimBackend", "Nim", "broad"),
    ("elixir", "ElixirBackend", "Elixir", "long"),
    ("haskell", "HaskellBackend", "Haskell", "long"),
    ("ocaml", "OCamlBackend", "OCaml", "long"),
    ("julia", "JuliaBackend", "Julia", "long"),
    ("r", "RBackend", "R", "long"),
    ("perl", "PerlBackend", "Perl", "long"),
    ("odin", "OdinBackend", "Odin", "long"),
    ("vlang", "VBackend", "V", "long"),
    ("chapel", "ChapelBackend", "Chapel", "long"),
    ("wasm", "WasmBackend", "WebAssembly", "broad"),
)


@dataclass
class Entry:
    name: str
    label: str
    tier: str
    factory: type


BROKEN: dict[str, str] = {}


def _load() -> dict[str, Entry]:
    entries: dict[str, Entry] = {}
    for module_name, class_name, label, tier in BACKENDS:
        try:
            module = importlib.import_module(f".backends.{module_name}", package=__package__)
        except Exception as error:  # noqa: BLE001
            # One backend that will not import must not stop the others from
            # being written, so the failure is recorded and the language is
            # reported as unavailable rather than taking the run down with it.
            BROKEN[module_name] = f"{type(error).__name__}: {error}"
            continue
        entries[module_name] = Entry(module_name, label, tier, getattr(module, class_name))
    return entries


REGISTRY = _load()


def names() -> list[str]:
    return [name for name, *_ in BACKENDS]


def available() -> list[str]:
    return [name for name in names() if name in REGISTRY]


def by_label(label: str) -> str:
    """The directory a language's bindings are written to."""
    for module_name, _cls, name, _tier in BACKENDS:
        if name == label:
            return module_name
    return label.lower().replace("+", "p").replace("#", "sharp").replace(" ", "").lower()


def build(name: str, abi, layout, options: Options) -> Backend:
    return REGISTRY[name].factory(abi, layout, options)


def tiers() -> dict[str, list[str]]:
    grouped: dict[str, list[str]] = {}
    for module_name, _cls, label, tier in BACKENDS:
        grouped.setdefault(tier, []).append(label)
    return grouped
