"""Generated bindings for the Louder engine, in many languages.

The package reads the C header `cbindgen` writes from `louder_rust`, works out
how everything in it is laid out, and hands the result to a backend per target
language. Nothing here knows about a particular language; that lives in
`louder_bindings.backends`.
"""

from __future__ import annotations

from .ir import Abi, Layout, Target

__all__ = ["Abi", "Layout", "Target", "parse_header", "DEFAULT_TARGET"]

from .cparse import parse_header

# The engine's own build is 64-bit on every desktop platform, and that is what
# the shared library and the tests are built for. WebAssembly is the one target
# where `size_t` and pointers are half the width, and the backend for it says so
# rather than having it inferred.
DEFAULT_TARGET = Target("native", pointer_size=8, size_t_size=8)
