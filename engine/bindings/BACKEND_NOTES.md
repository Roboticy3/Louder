# Writing a backend

A backend is one language. It reads the model and returns the files that make up
a binding for it. Nothing about the engine's boundary is written in a backend:
everything comes from `louder_bindings/ir.py`, which was built from
`louder_rust.h`, which `cbindgen` built from `engine/louder_rust/src/lib.rs`.

That is the whole design. There is exactly one description of the boundary, and
a backend is a way of printing it.

## The short version

```python
"""One or two sentences on why this language gets a backend at all."""

from __future__ import annotations

from ..ir import Func, Named, Pointer, Scalar, Struct, Type
from ..naming import Names, pascal, snake
from .base import Backend, Emitted


class ZBackend(Backend):
    name = "z"            # the directory under out/
    label = "Z"           # what it is called in the README table
    tier = "broad"        # how much polish it got; see below
    convenience = "thin"  # "none", "thin", or "full"
    needs_library = True  # copies the engine beside the bindings
    extension = "z"

    def make_names(self) -> Names:
        return Names("z", type_case=pascal, func_case=camel, field_case=snake)

    def type_expr(self, type: Type) -> str:
        ...   # a scalar, an aggregate, a pointer, an array: what this language calls it
```

That is enough for a declaration-only binding. `Backend.render()` then produces
types, constants, and function declarations in that order.

## What the model gives you

`self.abi` is an `Abi`:

| | |
| --- | --- |
| `abi.structs` | `name -> Struct`, each with `.fields` of `(name, type)` |
| `abi.unions` | `name -> TaggedUnion`, with `.tag_type` and `.variants` |
| `abi.union_bodies` | the struct names that hold a tagged union's arms |
| `abi.enums` | `name -> Enum`, with `.variants` of `(name, explicit-or-None)` |
| `abi.aliases` | `name -> Alias`, `.target` being the aliased type |
| `abi.opaques` | `name -> Opaque`, always behind a pointer |
| `abi.consts` | `name, type, value` |
| `abi.funcs` | `name, ret, params, doc` |
| `abi.bitflags` | `name -> Enum`, synthesized from a one-field struct plus its constants |

A `Type` is one of `Scalar`, `Named`, `Pointer` (with `.const`), or `Array`.
Resolve an alias before asking anything structural:

```python
resolved = self.abi.resolve(type)          # follows aliases to something with a size
if isinstance(resolved, Pointer): ...
```

`self.layout` answers sizes and offsets, and `self.layout.fields(struct)` gives
each field with the offset it sits at. Sizes are computed the way a C compiler
computes them and are cross-checked against `ctypes` on every run, so a backend
can trust them.

Two questions are worth knowing the answers to:

- `self.abi.is_aggregate(type)` — true for anything that is not a plain scalar.
  Every FFI asks this differently: ctypes wants a `Structure`, Go wants a copy,
  wasm32 wants a pointer to a copy in linear memory.
- `self.abi.union_arms(union)` — `(tag_value, variant_name, body_or_None)` for
  every arm of a tagged union. Pairing is by name, not position, because an enum
  with a valueless variant has fewer union members than it has variants.

## The model of this engine

Worth knowing before writing anything, because it decides what a binding has to
do:

- **`LouderName` is an `int32_t`.** Text crosses as bytes and comes back as a
  handle. Handles come from a hash, so they are **not sequential** and a binding
  must never guess one.
- **Strings are `(uint8_t *, size_t)`**, never `char *`.
- **The engine reports truncation rather than refusing it.** Every read returns
  how many bytes fitted. The only correct loop is: read, and if the answer
  filled the buffer, ask again with more room.
- **`declare_name` refuses to make a second id** for text that already has one,
  returning `LOUDER_NAME_NONE`. `get_named_id` is the lookup. A binding that
  offers one `name(text)` has to do both.
- **Sixteen cursors**, all the same shape: `start`, one result, an end. They are
  found by rule, not by list — any function whose last parameter is named `start`
  is one.
- **Four tagged unions** with 19, 2, 15 and 2 arms. A host switches on the tag.

## Naming

`Names(language, type_case=..., func_case=..., field_case=..., const_case=...,
arg_case=...)` takes functions from `louder_bindings/naming.py`: `snake`,
`camel`, `pascal`, `upper_snake`, `kebab`, `dotted`, or `keep` to leave the
header's own spelling alone. It also suffixes anything that collides with that
language's keywords, which are listed per language in `RESERVED`.

Give a backend `keep` when it writes something C-shaped, so `louder_variant` is
not standing where `LouderVariant` is.

## Files

Override `build` when there is more than one file:

```python
def build(self, out: Emitted) -> None:
    out.add_lines("louder.z", self.render())
    out.add("walk.z", self.runtime("z/walk.z").read_text(encoding="utf-8"))
    out.add_lines("README.md", self.render_readme())
```

`out.add` takes text, `out.add_binary` takes bytes. Anything in
`louder_bindings/runtime/<lang>/` is copied verbatim by `self.runtime(...)` —
that directory is for code that is about the language rather than about the
engine, which does not belong in a generator.

`louder_rust.h` has no prose in it unless `documentation` is on, and the
generator turns it on in the copy of the cbindgen config it writes to
`out/abi/`. That is why `func.doc` is worth carrying everywhere.

## Tiers

`registry.py` sorts the README table by tier, and the tier says how much a
language got:

- **deep** — a raw layer and a fully idiomatic one on top of it
- **solid** — complete bindings, compiled rather than reinterpreted
- **broad** — complete bindings with a thin convenience layer
- **long** — declarations only, for the tail of FFI-capable languages

A backend that raises `NotImplementedError` from `generate()` is reported as
skipped rather than stopping the run, which is the honest way to say a language
is not ready.

## House rules

These exist because breaking them costs real time:

1. **Quoting.** A Python string that will contain a `"` in its output is
   single-quoted: `'    the "x" case'`. And no apostrophes in generated doc text,
   because it ends up inside a Python literal.
2. **Newlines.** The generator writes with `newline="\n"`, so write your file
   with the `write` tool or `io.open(..., encoding='utf-8', newline='\n')`.
   `Set-Content` on PowerShell adds a BOM.
3. **Check it ran.** `python generate.py --lang <x> --no-wasm-build --header
   <header>` and read the output. A backend that imports and produces nothing is
   a failure, not a quiet success.
4. **Compile it.** Anything that compiles should be compiled. A declaration that
   does not build is not a binding.
5. **Say what a guess is.** Where a value cannot be derived from the header, the
   generated code says so in a comment rather than inventing one.