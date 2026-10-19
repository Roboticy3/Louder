"""The language-neutral description of the engine's C boundary.

Everything the generator knows comes from `louder_rust.h`: a set of scalar
types, tagged unions, plain structs, enums, constants, and `extern "C"`
functions. This module turns that into objects the backends can walk, and works
out how each one is laid out in memory.

The layout rules here are the ones C guarantees on every ABI the engine targets:
each type has a size and an alignment, fields sit at the next offset that is a
multiple of their own alignment, and a struct is as large as its fields rounded
up to its own alignment. `layout.check_against_ctypes` asserts that the result
matches what ctypes derives independently, because a binding whose offsets are
off by four bytes is wrong in a way no type checker will catch.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Iterable, Optional, Union

# --------------------------------------------------------------------------
# Types
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class Scalar:
    """A primitive: `void`, `bool`, `i32`, `size_t`, and so on."""

    name: str

    def __str__(self) -> str:
        return self.name


@dataclass(frozen=True)
class Named:
    """A name the header gives to something: a struct, enum, or alias."""

    name: str

    def __str__(self) -> str:
        return self.name


@dataclass(frozen=True)
class Pointer:
    to: "Type"
    # Whether the header spelled this `const`. Nothing downstream needs it to
    # work out a layout, but a language where the distinction is expressible
    # and worth keeping — Rust's `*const` — would otherwise lose it.
    const: bool = False

    def __str__(self) -> str:
        return ("*const " if self.const else "*") + str(self.to)


@dataclass(frozen=True)
class Array:
    of: "Type"
    length: int

    def __str__(self) -> str:
        return f"[{self.length}]{{{self.of}}}"


Type = Union[Scalar, Named, Pointer, Array]


VOID = Scalar("void")
BOOL = Scalar("bool")

# Sizes are in bytes and are the same everywhere the engine is built: 32-bit
# wasm, 64-bit Windows, and 64-bit Linux all agree on these.
SCALAR_INFO: dict[str, tuple[int, int]] = {
    "void": (0, 1),
    "bool": (1, 1),
    "int8_t": (1, 1),
    "uint8_t": (1, 1),
    "char": (1, 1),
    "int16_t": (2, 2),
    "uint16_t": (2, 2),
    "short": (2, 2),
    "int32_t": (4, 4),
    "uint32_t": (4, 4),
    "int": (4, 4),
    "unsigned": (4, 4),
    "float": (4, 4),
    "int64_t": (8, 8),
    "uint64_t": (8, 8),
    "long long": (8, 8),
    "double": (8, 8),
}

# `size_t` and pointers are the only things whose width follows the target. The
# generator is told which it is looking at rather than guessing, because getting
# it wrong silently truncates a length or mangles a pointer.
SIZE_T = Scalar("size_t")
SSIZE_T = Scalar("ptrdiff_t")


@dataclass(frozen=True)
class Target:
    """The ABI the bindings are being written for."""

    name: str
    pointer_size: int = 8
    size_t_size: int = 8

    @property
    def pointer_align(self) -> int:
        return self.pointer_size

    def scalar_info(self, scalar: Scalar) -> tuple[int, int]:
        if scalar is SIZE_T or scalar is SSIZE_T:
            return (self.size_t_size, self.size_t_size)
        if scalar.name in ("intptr_t", "uintptr_t"):
            return (self.pointer_size, self.pointer_size)
        try:
            return SCALAR_INFO[scalar.name]
        except KeyError:
            raise UnknownType(f"no size is known for `{scalar.name}`") from None


class UnknownType(Exception):
    """Raised when the header contains something the model does not describe."""


# --------------------------------------------------------------------------
# Declarations
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class Param:
    name: str
    type: Type
    doc: str = ""


@dataclass(frozen=True)
class Func:
    name: str
    ret: Type
    params: tuple[Param, ...]
    doc: str = ""
    # Filled in by `wasm.resolve` when the module is built for wasm32. Every
    # other target leaves this None and lets the language's own FFI rules apply.
    wasm: Optional["WasmSignature"] = None

    @property
    def doc_params(self) -> list[Param]:
        return [p for p in self.params if p.name != "ctx"]


@dataclass(frozen=True)
class Field:
    name: str
    type: Type


@dataclass(frozen=True)
class Variant:
    """One arm of a tagged union: the member the C header calls it."""

    name: str
    type: Type


@dataclass
class Struct:
    name: str
    fields: tuple[Field, ...]
    doc: str = ""

    def field(self, name: str) -> Field:
        for f in self.fields:
            if f.name == name:
                return f
        raise KeyError(f"{self.name} has no field `{name}`")


@dataclass
class TaggedUnion:
    """A `#[repr(C)]` enum: a tag, then a union of the arms' bodies.

    cbindgen emits the body of each arm as an anonymous struct with one field,
    named after the arm in snake case. A host cannot build one of these without
    knowing that layout, which is why the engine exports `make_*` functions for
    every arm; a binding only ever has to read one.
    """

    name: str
    tag_type: Named
    variants: tuple[Variant, ...]
    doc: str = ""

    @property
    def tag_enum(self) -> "Enum | None":
        return None  # filled in by Abi.link()

    def variant(self, name: str) -> Variant:
        for v in self.variants:
            if v.name == name:
                return v
        raise KeyError(f"{self.name} has no variant `{name}`")


@dataclass
class Enum:
    name: str
    variants: tuple[tuple[str, Optional[int]], ...]
    # cbindgen hides the underlying type behind a `__STDC_VERSION__` guard, so
    # the header does not say what an `enum` is. Rust's `repr` does, and the
    # two always agree for the enums here: an untagged `enum` is 4 bytes unless
    # it is explicitly `repr(u8)`.
    underlying: Type = field(default_factory=lambda: Scalar("int32_t"))
    doc: str = ""

    def value(self, name: str) -> int:
        for variant, value in self.variants:
            if variant == name:
                return 0 if value is None else value
        raise KeyError(f"{self.name} has no variant `{name}`")

    @property
    def names(self) -> list[str]:
        return [v for v, _ in self.variants]

    def next_value(self, after: str) -> int:
        seen = [0 if v is None else v for _, v in self.variants]
        index = self.names.index(after)
        return seen[index + 1] if index + 1 < len(seen) else seen[-1] + 1


@dataclass
class Alias:
    name: str
    target: Type
    doc: str = ""


@dataclass
class Opaque:
    name: str
    doc: str = ""


@dataclass(frozen=True)
class Const:
    name: str
    type: Type
    value: int | float


@dataclass
class Abi:
    """Everything the header declares, resolved and cross-linked."""

    structs: dict[str, Struct] = field(default_factory=dict)
    unions: dict[str, TaggedUnion] = field(default_factory=dict)
    enums: dict[str, Enum] = field(default_factory=dict)
    aliases: dict[str, Alias] = field(default_factory=dict)
    opaques: dict[str, Opaque] = field(default_factory=dict)
    consts: list[Const] = field(default_factory=list)
    funcs: list[Func] = field(default_factory=list)
    # Structs the parser invented to hold the arms of a tagged union. They are
    # alternatives rather than a sequence, so a backend lays them out as a union
    # instead of following the fields in order.
    union_bodies: set[str] = field(default_factory=set)

    def link(self) -> None:
        """Resolve the tag enums of tagged unions, and check nothing dangles.

        Everything downstream assumes a `TaggedUnion`'s tag names an `Enum` that
        this same `Abi` declares: the ergonomic layers build a case per variant
        off the tag's ordinal, and a missing enum would mean a silent mismatch
        between the two.

        The documentation is settled here too. `cbindgen` puts an enum's prose on
        the tag it generates rather than on the struct around it, and it is the
        same prose either way, so a union with no words of its own borrows them.
        """
        for union in self.unions.values():
            tag = union.tag_type.name
            if tag not in self.enums:
                raise UnknownType(f"the tag of `{union.name}` is `{tag}`, which is not declared")
            if not union.doc and self.enums[tag].doc:
                union.doc = self.enums[tag].doc

        for alias in self.aliases.values():
            if not alias.doc:
                target = alias.target
                if isinstance(target, Named):
                    alias.doc = self.doc_of(target.name)

        for opaque in self.opaques.values():
            if not opaque.doc:
                opaque.doc = f"The handle to {opaque.name}, obtained when the engine starts."

    def doc_of(self, name: str) -> str:
        for group in (self.structs, self.unions, self.enums, self.aliases):
            found = group.get(name)
            if found is not None:
                return found.doc
        return ""

    def func(self, name: str) -> Func:
        for f in self.funcs:
            if f.name == name:
                return f
        raise KeyError(f"no function called `{name}`")

    def has_func(self, name: str) -> bool:
        return any(f.name == name for f in self.funcs)

    def resolve(self, type: Type) -> Type:
        """Follow aliases until something with a size turns up."""
        seen = 0
        while isinstance(type, Named):
            alias = self.aliases.get(type.name)
            if alias is None:
                return type
            type = alias.target
            seen += 1
            if seen > 32:
                raise UnknownType(f"alias chain from `{type}` does not end")
        return type

    def is_aggregate(self, type: Type) -> bool:
        """True for anything that is not a plain scalar.

        This is the question every FFI asks differently. ctypes wants a
        `Structure`, cgo wants a copy, and wasm32 wants a pointer to a copy in
        linear memory, so the backends ask it and then apply their own rule.
        """
        type = self.resolve(type)
        if isinstance(type, Pointer):
            return False
        if isinstance(type, Array):
            return True
        if isinstance(type, Named):
            return True
        return type is not VOID and type.name not in SCALAR_INFO and type.name not in ("size_t", "ptrdiff_t")

    def is_union_body(self, type: Type) -> bool:
        """True for the struct a parser invented to hold a union's arms.

        Its fields are alternatives laid on top of each other, not a sequence,
        so a backend that would otherwise emit them one after another has to
        know to overlap them instead.
        """
        name = type.name if isinstance(type, Named) else None
        if name is None:
            return False
        if name in self.union_bodies:
            return True
        resolved = self.resolve(type)
        return isinstance(resolved, Named) and resolved.name in self.union_bodies

    def union_arms(self, union: TaggedUnion) -> list[tuple[int, str, "Variant | None"]]:
        """Every arm of a tagged union, by tag value.

        Each entry is the value the tag takes, the name the engine gave the arm,
        and the body it carries — which is `None` for an arm that carries
        nothing, since `cbindgen` writes no union member for one of those.

        The pairing is by name rather than by position, and tolerates the
        trailing underscore `cbindgen` adds where the snake-case name would be
        a C keyword: the arm `Int` is the member `int_`, because `int` cannot
        be a field name in C. Counting along the list instead lands on the
        wrong arm as soon as an enum has a valueless variant or a keyword
        clash, which is most of them.
        """
        from .naming import snake

        tag = self.enums.get(union.tag_type.name)
        if tag is None:
            return []

        by_field = {variant.name: variant for variant in union.variants}
        arms: list[tuple[int, str, Variant | None]] = []
        for value, (name, _explicit) in enumerate(tag.variants):
            spelled = snake(name)
            variant = by_field.get(spelled) or by_field.get(spelled.rstrip("_") + "_")
            arms.append((value, name, variant))
        return arms

    def arm_of(self, union: TaggedUnion, value: int) -> tuple[str, "Variant | None"]:
        """The arm a tag value names, and what it carries."""
        for tag_value, name, variant in self.union_arms(union):
            if tag_value == value:
                return name, variant
        raise KeyError(f"{union.name} has no arm with the tag {value}")

    def bitflag(self, name: str) -> Enum | None:
        """The enum behind a flags struct, if it looks like one.

        cbindgen lowers a `bitflags` type to a struct wrapping the bits and
        emits one `Type_NAME` constant per flag. A binding can turn that back
        into an `IntFlag` or an enum with `|`, which is what makes passing flags
        readable at the call site.
        """
        struct = self.structs.get(name)
        if struct is None or len(struct.fields) != 1:
            return None
        bits = struct.fields[0]
        if not isinstance(self.resolve(bits.type), Scalar) or self.resolve(bits.type).name not in (
            "int32_t",
            "uint32_t",
            "int64_t",
            "uint64_t",
            "uint8_t",
            "uint16_t",
        ):
            return None
        prefix = name + "_"
        flags = [c for c in self.consts if c.name.startswith(prefix) and isinstance(c.value, int)]
        if not flags:
            return None
        return Enum(
            name=name,
            variants=tuple((c.name[len(prefix):], int(c.value)) for c in flags),
            underlying=self.resolve(bits.type),
            doc=f"The flags of a {name[:-5].lower() if name.endswith('Flags') else name}.",
        )

    @property
    def bitflags(self) -> dict[str, Enum]:
        return {name: e for name in self.structs if (e := self.bitflag(name)) is not None}


# --------------------------------------------------------------------------
# Layout
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class FieldLayout:
    name: str
    type: Type
    offset: int
    size: int


class Layout:
    """Sizes and offsets, computed the way a C compiler computes them."""

    def __init__(self, abi: Abi, target: Target):
        self.abi = abi
        self.target = target
        self._sizes: dict[object, tuple[int, int]] = {}

    def size_align(self, type: Type) -> tuple[int, int]:
        # A declaration passed whole, rather than named, is still answerable:
        # the backends walk structs and unions directly often enough that
        # making every caller wrap them in a `Named` would only invite mistakes.
        if isinstance(type, Struct):
            return self.struct_size_align(type)
        if isinstance(type, TaggedUnion):
            return self.union_size_align(type)
        if isinstance(type, Enum):
            return self.size_align(type.underlying)
        if isinstance(type, Alias):
            return self.size_align(type.target)
        if isinstance(type, Opaque):
            # Only ever handled behind a pointer; the size is never asked for.
            return (0, 1)
        type = self.abi.resolve(type)
        if isinstance(type, Scalar):
            return self.target.scalar_info(type)
        if isinstance(type, Pointer):
            return (self.target.pointer_size, self.target.pointer_align)
        if isinstance(type, Array):
            size, align = self.size_align(type.of)
            return (size * type.length, align)
        if isinstance(type, Named):
            return self.named_size_align(type.name)

        raise UnknownType(f"cannot size `{type}`")

    def struct_size_align(self, struct: Struct) -> tuple[int, int]:
        key = ("struct", struct.name)
        if key in self._sizes:
            return self._sizes[key]
        self._sizes[key] = (0, 1)

        align = 1
        size = 0
        for f in struct.fields:
            fsize, falign = self.size_align(f.type)
            align = max(align, falign)
            if self.abi.is_union_body(Named(struct.name)):
                size = max(size, fsize)
            else:
                size = _round_up(size, falign) + fsize

        result = (_round_up(size, align), align)
        self._sizes[key] = result
        return result

    def union_size_align(self, union: TaggedUnion) -> tuple[int, int]:
        key = ("union", union.name)
        if key in self._sizes:
            return self._sizes[key]
        self._sizes[key] = (0, 1)

        # The tag sits in front of the bodies and the whole thing is aligned to
        # the widest body, which is what makes `sizeof` carry the body's padding
        # rather than the tag's.
        tag_size, _ = self.size_align(union.tag_type)
        align = 1
        size = 0
        for variant in union.variants:
            vsize, valign = self.size_align(variant.type)
            size = max(size, vsize)
            align = max(align, valign)
        body_offset = _round_up(tag_size, align)
        result = (_round_up(body_offset + size, align), align)
        self._sizes[key] = result
        return result

    def size(self, type: Type) -> int:
        return self.size_align(type)[0]

    def named_size_align(self, name: str) -> tuple[int, int]:
        if name in self._sizes:
            return self._sizes[name]

        # A struct that refers to itself through a pointer still has a size, so
        # the placeholder has to be in place before the fields are walked.
        self._sizes[name] = (0, 1)
        result = self._compute(name)
        self._sizes[name] = result
        return result

    def _compute(self, name: str) -> tuple[int, int]:
        struct = self.abi.structs.get(name)
        if struct is not None:
            return self.struct_size_align(struct)

        union = self.abi.unions.get(name)
        if union is not None:
            return self.union_size_align(union)

        alias = self.abi.aliases.get(name)
        if alias is not None:
            return self.size_align(alias.target)

        enum = self.abi.enums.get(name)
        if enum is not None:
            return self.size_align(enum.underlying)

        if name in self.abi.opaques:
            # Only ever handled behind a pointer; the size is never asked for.
            return (0, 1)

        raise UnknownType(f"`{name}` is declared in the header but not described here")

    def fields(self, struct: Struct) -> list[FieldLayout]:
        """Where each field of `struct` sits.

        A union body is the one struct whose fields are alternatives rather than
        a sequence, so every one of them starts at zero.
        """
        overlapping = self.abi.is_union_body(Named(struct.name))
        out: list[FieldLayout] = []
        offset = 0
        for f in struct.fields:
            size, align = self.size_align(f.type)
            if overlapping:
                out.append(FieldLayout(f.name, f.type, 0, size))
                continue
            offset = _round_up(offset, align)
            out.append(FieldLayout(f.name, f.type, offset, size))
            offset += size
        return out

    def tag_offset(self, union: TaggedUnion) -> int:
        return 0

    def variant_offset(self, union: TaggedUnion, variant: Variant) -> int:
        tag_size, _ = self.size_align(union.tag_type)
        _, align = self.size_align(union)
        return _round_up(tag_size, align)

    def check_against_ctypes(self) -> list[str]:
        """Compare every computed layout with ctypes' independent derivation.

        ctypes works the layout out from the same declarations the backends are
        given, so the two agreeing is a real check on this file and not a
        tautology. Anything it reports is a bug here, not a disagreement to
        paper over.
        """
        try:
            import ctypes
        except ImportError:  # pragma: no cover - ctypes is in the stdlib
            return []

        from .ctypes_layout import build_ctypes

        problems: list[str] = []
        declared, _ = build_ctypes(self.abi, self)
        for name, struct in self.abi.structs.items():
            if name not in declared:
                problems.append(f"{name} was not built for the check")
                continue
            built = declared[name]
            if ctypes.sizeof(built) != self.size(struct):
                problems.append(
                    f"{name}: size {ctypes.sizeof(built)} vs computed {self.size(struct)}"
                )
            if self.abi.is_union_body(struct):
                # Checked through the arm offsets below instead.
                continue
            for computed in self.fields(struct):
                actual = getattr(built, computed.name).offset
                if actual != computed.offset:
                    problems.append(
                        f"{name}.{computed.name}: offset {actual} vs computed {computed.offset}"
                    )
        for name, union in self.abi.unions.items():
            if name not in declared:
                problems.append(f"{name} was not built for the check")
                continue
            if ctypes.sizeof(declared[name]) != self.size(union):
                problems.append(
                    f"{name}: size {ctypes.sizeof(declared[name])} vs computed {self.size(union)}"
                )
            if not union.variants:
                continue
            # The arms all sit at zero inside the body union, so what is worth
            # comparing is where that union itself starts within the tagged
            # union, which is the only offset this model has an opinion about.
            expected = self.variant_offset(union, union.variants[0])
            actual = getattr(declared[name], "body").offset
            if expected != actual:
                problems.append(f"{name}: bodies start at {actual} vs computed {expected}")
        return problems


def _round_up(value: int, align: int) -> int:
    if align <= 1:
        return value
    return (value + align - 1) // align * align


# --------------------------------------------------------------------------
# WebAssembly
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class WasmSignature:
    """How a function actually appears in the compiled module.

    wasm32 has no caller-cleaned registers and no hidden return slot in the C
    sense, so the LLVM WebAssembly ABI does two things that a binding has to
    know about:

    - an aggregate larger than a single register is not returned in one, so a
      pointer to caller-owned space is passed as the *first* argument and the
      function returns nothing;
    - an aggregate passed by value is copied into linear memory by the caller,
      so it arrives as a pointer too.

    Both are recorded here rather than re-derived, because the module is right
    there and the ABI rules are the sort of thing that changes.
    """

    params: tuple[str, ...]
    results: tuple[str, ...]
    # Index of the argument that receives an aggregate return, or None when the
    # return value came back in registers.
    sret_index: Optional[int]
    # Argument indices whose IR parameter is an aggregate passed by value.
    by_value: dict[int, str]

    def param_type(self, index: int) -> str:
        return self.params[index]
