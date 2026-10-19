"""Turning the model into `ctypes` declarations.

The generated Python module is one long list of `Structure`, `Union` and
pointer declarations, and this is the part of it that has no language in it:
every type in the header becomes a `ctypes` type, in an order where each one is
defined after everything it contains.

It is also what `Layout` checks itself against. `ctypes` derives offsets from
the same declarations the backends are handed, so the two arriving at the same
answer is evidence about this generator rather than a restatement of it.
"""

from __future__ import annotations

import ctypes
from typing import Any

from .ir import (
    Abi,
    Array,
    Field,
    Named,
    Pointer,
    Scalar,
    Struct,
    TaggedUnion,
    Type,
)

_CTYPES_SCALARS: dict[str, Any] = {
    "void": None,
    "bool": ctypes.c_bool,
    "int8_t": ctypes.c_int8,
    "uint8_t": ctypes.c_uint8,
    "int16_t": ctypes.c_int16,
    "uint16_t": ctypes.c_uint16,
    "int32_t": ctypes.c_int32,
    "uint32_t": ctypes.c_uint32,
    "int64_t": ctypes.c_int64,
    "uint64_t": ctypes.c_uint64,
    "float": ctypes.c_float,
    "double": ctypes.c_double,
}


def _scalar(name: str, size: int) -> Any:
    fixed = _CTYPES_SCALARS.get(name)
    if fixed is not None:
        return fixed
    if name in ("size_t", "ptrdiff_t", "intptr_t", "uintptr_t"):
        return ctypes.c_uint64 if size == 8 else ctypes.c_uint32
    raise KeyError(f"`{name}` has no ctypes equivalent")


class CtypesBuilder:
    """Builds a ctypes type for every name the header declares."""

    def __init__(self, abi: Abi, layout):
        self.abi = abi
        self.layout = layout
        self.types: dict[str, Any] = {}
        # Bodies declared after everything they contain, which is what ctypes
        # needs before a `Structure` will point at them.
        self.order: list[str] = []

    # -- resolution -------------------------------------------------------

    def type_for(self, type: Type) -> Any:
        if isinstance(type, Pointer):
            inner = self.type_for(type.to)
            if isinstance(inner, ctypes._CArgObject):  # pragma: no cover
                return ctypes.c_void_p
            return ctypes.POINTER(inner)

        if isinstance(type, Array):
            return self.type_for(type.of) * type.length

        if isinstance(type, Scalar):
            if type.name == "void":
                return None
            return _scalar(type.name, self.layout.size(type))

        if isinstance(type, Named):
            return self.named(type.name)

        raise KeyError(f"no ctypes type for {type!r}")

    def named(self, name: str) -> Any:
        if name in self.types:
            return self.types[name]

        enum = self.abi.enums.get(name)
        if enum is not None:
            # The raw type, not a subclass of it. A subclass hands back an
            # instance that does not compare equal to the integers a tag is
            # switched on, and the ergonomic layer is where an actual `IntEnum`
            # is worth building.
            built = _scalar(enum.underlying.name, self.layout.size(enum.underlying))
            self.types[name] = built
            return built

        alias = self.abi.aliases.get(name)
        if alias is not None:
            built = self.type_for(alias.target)
            self.types[name] = built
            return built

        if name in self.abi.structs:
            built = self.build_struct(self.abi.structs[name])
            self.types[name] = built
            self.order.append(name)
            return built

        if name in self.abi.unions:
            built = self.build_union(self.abi.unions[name])
            self.types[name] = built
            self.order.append(name)
            return built

        if name in self.abi.opaques:
            # Never dereferenced, so its contents do not matter. The size is the
            # pointer's, which is the only size anyone asks for.
            built = type(name, (ctypes.Structure,), {"__doc__": f"An opaque handle to {name}."})
            self.types[name] = built
            return built

        raise KeyError(f"`{name}` is not described by the model")

    # -- aggregates -------------------------------------------------------

    def build_struct(self, struct: Struct) -> Any:
        overlapping = self.abi.is_union_body(Named(struct.name))
        base = ctypes.Union if overlapping else ctypes.Structure

        # Two-tuples only: ctypes reads a third element as a bit width. Letting
        # it place the fields itself is the point, since its answer is what
        # `Layout` is measured against.
        members = [(f.name, self.type_for(f.type)) for f in struct.fields]

        # No `_anonymous_` here. ctypes only offers it for members that are
        # themselves structs or unions, and these are plain integers and floats
        # sitting on top of each other; they are reached by name already.
        return type(struct.name, (base,), {"_fields_": members, "__doc__": self.doc_for(struct)})

    def build_union(self, union: TaggedUnion) -> Any:
        """A tag, then the arms overlapped on top of each other.

        ctypes has no anonymous union inside a struct, so the arms go in a union
        of their own and the struct points at it anonymously. That is also what
        makes `event.data.regular_motion` read the way the C++ header does,
        with no accessor of this generator's own in the way.
        """
        body = self.types.get(f"{union.name}_Body") or self.named(f"{union.name}_Body")
        tag = self.type_for(union.tag_type)
        built = type(
            union.name,
            (ctypes.Structure,),
            {
                "_fields_": [("tag", tag), ("body", body)],
                "_anonymous_": ("body",),
                "__doc__": self.doc_for(union),
            },
        )
        return built

    def doc_for(self, decl) -> str:
        doc = (decl.doc or "").strip()
        return doc or f"The `{decl.name}` type."


def build_ctypes(abi: Abi, layout) -> tuple[dict[str, Any], list[str]]:
    """Build every declared type, in an order each can be declared in."""
    builder = CtypesBuilder(abi, layout)

    # Anything reachable from a field has to exist first. Walking the
    # declarations in dependency order is what makes `_fields_` legal.
    for struct in sorted(abi.structs.values(), key=lambda s: _depth(abi, Named(s.name))):
        builder.named(struct.name)
    for name in sorted(abi.unions):
        builder.named(name)
    for name in sorted(abi.enums):
        builder.named(name)
    for name in sorted(abi.aliases):
        builder.named(name)
    for name in sorted(abi.opaques):
        builder.named(name)

    return builder.types, builder.order


def _depth(abi: Abi, type: Type, seen: frozenset[str] = frozenset()) -> int:
    """How deep a type's dependencies go, used to order declarations."""
    best = 0
    if isinstance(type, Named):
        if type.name in seen:
            return 0
        seen = seen | {type.name}
        for field in _fields_of(abi, type.name):
            best = max(best, 1 + _depth(abi, field.type, seen))
    if isinstance(type, Pointer):
        best = max(best, _depth(abi, type.to, seen))
    if isinstance(type, Array):
        best = max(best, _depth(abi, type.of, seen))
    return best


def _fields_of(abi: Abi, name: str) -> list[Any]:
    struct = abi.structs.get(name)
    if struct is not None:
        return list(struct.fields)

    union = abi.unions.get(name)
    if union is not None:
        return list(abi.structs[union.name + "_Body"].fields)

    alias = abi.aliases.get(name)
    if alias is not None:
        return [Field("_", alias.target)]

    return []


def union_body_names(abi: Abi) -> list[str]:
    return sorted(abi.union_bodies)


def struct_list(abi: Abi) -> list[Struct]:
    return [abi.structs[name] for name in sorted(abi.structs)]
