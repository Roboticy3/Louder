"""The part of a C-shaped backend that is not about the calling convention.

C, C++, and the WebAssembly header all say the same types the same way, and all
three run into the same problem doing it: C has one namespace for everything, so
an enum constant called `ChoiceAppearance` and a struct of the same name cannot
both exist. The engine's own header has exactly that collision, which means a
header copied from it does not compile. The members are disambiguated here
instead, and only where they actually collide, so the rest keep the names the
engine's own documentation uses.

What is left over is the calling convention, which is the only thing that
actually differs between these three: C calls through a header, C++ calls
through a header and a wrapper, and WebAssembly calls through the module's own
type section.
"""

from __future__ import annotations

from typing import Optional

from ..ir import (
    Abi,
    Alias,
    Array,
    Enum,
    Func,
    Named,
    Opaque,
    Pointer,
    Scalar,
    Struct,
    TaggedUnion,
    Type,
    VOID,
)
from ..naming import Names, doc_lines, keep
from .base import Backend, Emitted

_SCALARS = {
    "void": "void",
    "bool": "bool",
    "int8_t": "int8_t",
    "uint8_t": "uint8_t",
    "char": "uint8_t",
    "int16_t": "int16_t",
    "uint16_t": "uint16_t",
    "short": "int16_t",
    "int32_t": "int32_t",
    "uint32_t": "uint32_t",
    "int": "int32_t",
    "unsigned": "uint32_t",
    "float": "float",
    "int64_t": "int64_t",
    "uint64_t": "uint64_t",
    "long long": "int64_t",
    "double": "double",
    "size_t": "size_t",
    "ptrdiff_t": "ptrdiff_t",
    "intptr_t": "intptr_t",
    "uintptr_t": "uintptr_t",
}

# Which C spelling an enum's underlying type gets. cbindgen hides the underlying
# type behind a `__STDC_VERSION__` guard, so the model carries it from Rust.
_ENUM_WIDTH = {
    "int32_t": "int32_t",
    "uint32_t": "uint32_t",
    "int8_t": "int8_t",
    "uint8_t": "uint8_t",
    "int16_t": "int16_t",
    "uint16_t": "uint16_t",
    "int64_t": "int64_t",
    "uint64_t": "uint64_t",
}


class ClikeBackend(Backend):
    """Types, constants, and the rules for spelling them in C."""

    name = "c"
    label = "C"
    tier = "broad"
    extension = "h"
    include_guard = "LOUDER_H"
    # Whether the declarations are `extern` or `static inline`. A header of
    # `extern` declarations is what the engine itself ships.
    externs = True
    # The name of the header as other files will include it.
    header_file = "louder.h"
    # What the header has to include for the types it uses, and for whatever
    # the convenience layer calls.
    system_includes = ("<stdbool.h>", "<stddef.h>", "<stdint.h>")
    # When set, replaces the standard banner. A backend whose header needs to
    # say something specific about how it was produced writes the whole comment
    # here, opener and closer included.
    own_banner: Optional[list[str]] = None

    def __init__(self, abi: Abi, layout, options):
        super().__init__(abi, layout, options)
        # The header's own spelling, in every position. A C-shaped binding is
        # read next to the header it came from, and `louder_variant` sitting
        # where `LouderVariant` does is one more thing to translate in the head.
        self.names = Names("c", type_case=keep, func_case=keep, field_case=keep,
                           const_case=keep, arg_case=keep, suffix="_")
        self._taken_names: set[str] | None = None

    # -- names -------------------------------------------------------------

    def taken_names(self) -> set[str]:
        """Every identifier the header declares as a type.

        C keeps struct tags, typedef names, enum constants, and ordinary
        identifiers in one namespace, so a name declared twice anywhere in the
        file is an error rather than a shadow. The enum members are the ones that
        collide in practice, because a variant of a tagged union and the struct
        it carries are named after the same thing.
        """
        if self._taken_names is None:
            taken: set[str] = set()
            for group in (self.abi.structs, self.abi.unions, self.abi.enums, self.abi.aliases, self.abi.opaques):
                taken.update(group)
            self._taken_names = taken
        return self._taken_names

    def enum_member(self, enum_name: str, variant: str) -> str:
        """The name an enum constant gets in this header.

        The engine's spelling is used wherever it can be. It cannot be used when
        the constant would redeclare something else in the file, or when it is a
        C keyword, and then the enum's own name goes in front of it. That is the
        spelling cbindgen produces when it has to disambiguate, so it is the one
        a reader is most likely to have seen.
        """
        member = variant + "_" if variant in self.names.reserved else variant
        if member not in self.taken_names():
            return member
        return enum_name + "_" + member

    def none_constant(self, type) -> str:
        """The negative value an id of this type takes when there is none.

        `LouderName` and `LouderActor` are both `int32_t` under different names,
        and the header gives each a `..._NONE` constant. Finding it by type rather
        than hard-coding it means an alias added later is picked up without this
        being told, and a walk over something with no constant falls back to the
        value the engine actually uses, which every one of them shares.
        """
        resolved = self.abi.resolve(type) if not hasattr(type, "name") else type
        if isinstance(resolved, Named):
            stem = "_".join(w.upper() for w in resolved.name.split("_") if w != "Louder")
            for const in self.abi.consts:
                if const.name.endswith("_" + stem + "_NONE") and isinstance(const.value, int) and const.value < 0:
                    return const.name
        return "-1"

    # -- types -------------------------------------------------------------

    def type_expr(self, type: Type, *, deref: bool = False) -> str:
        resolved = self.abi.resolve(type)
        if isinstance(resolved, Pointer):
            inner = self.type_expr(resolved.to)
            if inner.endswith("*"):
                inner = "(" + inner + ")"
            return ("const " + inner + "*") if resolved.const else (inner + "*")
        if isinstance(resolved, Array):
            inner = self.type_expr(resolved.of)
            if inner.endswith("*"):
                inner = "(" + inner + ")"
            return f"{inner}[{resolved.length}]"
        if isinstance(resolved, Scalar):
            try:
                return _SCALARS[resolved.name]
            except KeyError:
                raise NotImplementedError("no C spelling for " + resolved.name) from None
        if isinstance(resolved, Named):
            return self.names.type(resolved.name)
        if resolved is VOID:
            return "void"
        raise NotImplementedError("cannot spell " + str(type))

    # -- declarations ------------------------------------------------------

    def comment(self, text: str, indent: str = "") -> list[str]:
        return ["/*"] + [indent + line.rstrip() for line in doc_lines(text, " * ")] + [" */"]

    def is_body(self, name: str) -> bool:
        return name in self.abi.union_bodies

    def aggregate_order(self) -> list[str]:
        """Aggregates in an order C will accept, dependencies first.

        `declaration_order` also carries enums through, and an enum here is a
        typedef that has to precede anything naming it, so the enums are left
        out and written separately above.
        """
        return [name for name in self.declaration_order() if name not in self.abi.enums]

    def emit_enum(self, enum: Enum) -> list[str]:
        name = self.names.type(enum.name)
        underlying = enum.underlying.name if isinstance(enum.underlying, Scalar) else "int32_t"
        lines: list[str] = []
        if enum.doc:
            lines.extend(self.comment(enum.doc))
        lines.append("typedef " + _ENUM_WIDTH.get(underlying, "int32_t") + " " + name + ";")
        # The tag is not the typedef: in C++ an enum injects its own name into the
        # ordinary namespace, so `enum X` next to `typedef int32_t X` is an error
        # there even though C accepts it. A tag of its own sidesteps both.
        lines.append("enum " + name + "_values {")
        value = 0
        for variant, explicit in enum.variants:
            value = explicit if explicit is not None else value
            lines.append("    " + self.enum_member(name, variant) + " = " + str(value) + ",")
            value += 1
        lines.append("};")
        return lines

    def emit_struct(self, struct: Struct) -> list[str]:
        name = self.names.type(struct.name)
        lines: list[str] = []
        if struct.doc:
            lines.extend(self.comment(struct.doc))
        lines.append("typedef struct " + name + " {")
        for placed in self.layout.fields(struct):
            lines.append("    " + self.type_expr(placed.type) + " " + self.names.field(placed.name) + ";")
        lines.append("} " + name + ";")
        return lines

    def emit_body(self, struct: Struct) -> list[str]:
        """The type holding a tagged union's arms.

        Every field starts at zero, so the members overlap rather than follow
        each other. cbindgen writes these inline; giving them a name is what lets
        the union in front of them point at them.
        """
        name = self.names.type(struct.name)
        lines: list[str] = []
        if struct.doc:
            lines.extend(self.comment(struct.doc))
        lines.append("typedef union {")
        for placed in self.layout.fields(struct):
            lines.append("    " + self.type_expr(placed.type) + " " + self.names.field(placed.name) + ";")
        lines.append("} " + name + ";")
        return lines

    def emit_union(self, union: TaggedUnion) -> list[str]:
        name = self.names.type(union.name)
        body = self.names.type(union.name + "_Body")
        lines: list[str] = []
        if union.doc:
            lines.extend(self.comment(union.doc))
        lines.extend(
            [
                "typedef struct {",
                "    " + self.type_expr(union.tag_type) + " tag;",
                "    " + body + " body;",
                "} " + name + ";",
            ]
        )
        return lines

    def emit_opaque(self, opaque: Opaque) -> list[str]:
        name = self.names.type(opaque.name)
        lines: list[str] = []
        if opaque.doc:
            lines.extend(self.comment(opaque.doc))
        lines.append("typedef struct " + name + " " + name + ";")
        return lines

    def emit_alias(self, alias: Alias) -> list[str]:
        return ["typedef " + self.type_expr(alias.target) + " " + self.names.type(alias.name) + ";"]

    def emit_constants(self) -> list[str]:
        lines: list[str] = []
        for const in self.abi.consts:
            # A bitflag constant is a one-field struct, which C cannot write as a
            # literal, so it is spelled as the one field it wraps.
            target = self.abi.resolve(const.type)
            if isinstance(target, Named) and target.name in self.abi.structs:
                struct = self.abi.structs[target.name]
                if len(struct.fields) == 1:
                    field = struct.fields[0]
                    lines.append(
                        "#define "
                        + const.name
                        + " (("
                        + self.type_expr(field.type)
                        + ")("
                        + literal(self.abi.resolve(field.type), const.value)
                        + "))"
                    )
                    continue
            lines.append(
                "#define "
                + const.name
                + " (("
                + self.type_expr(const.type)
                + ")("
                + literal(target, const.value)
                + "))"
            )
        return lines

    def function_signature(self, func: Func) -> tuple[str, str]:
        """The return type and the argument list, as C spells them."""
        params = []
        for index, param in enumerate(func.params):
            params.append(self.type_expr(param.type) + " " + self.names.arg(param.name, index))
        return self.type_expr(func.ret), (", ".join(params) if params else "void")

    def emit_function(self, func: Func) -> list[str]:
        returns, arguments = self.function_signature(func)
        call = self.names.func(func.name) + "(" + arguments + ")"
        prefix = "extern " if self.externs else "static inline "
        one_line = prefix + returns + " " + call + ";"
        lines: list[str] = []
        if func.doc:
            lines.extend(self.comment(func.doc))
        if len(one_line) <= 100:
            lines.append(one_line)
            return lines
        lines.append(prefix + returns + " " + self.names.func(func.name) + "(")
        for position, param in enumerate(arguments.split(", ")):
            tail = "," if position + 1 < len(arguments.split(", ")) else ""
            lines.append("    " + param + tail)
        lines.append(");")
        return lines

    # -- the file ----------------------------------------------------------

    def render(self) -> list[str]:
        lines: list[str] = []
        lines.extend(self.banner_block())
        lines.append("")
        lines.append("#ifndef " + self.include_guard)
        lines.append("#define " + self.include_guard)
        lines.append("")
        for include in self.system_includes:
            lines.append("#include " + include)
        lines.append("")
        lines.extend(self.preamble())

        for enum in self.ordered_enums():
            lines.extend(self.emit_enum(enum))
            lines.append("")

        for name in self.aggregate_order():
            if self.is_body(name):
                lines.extend(self.emit_body(self.abi.structs[name]))
            elif name in self.abi.unions:
                lines.extend(self.emit_union(self.abi.unions[name]))
            elif name in self.abi.structs:
                lines.extend(self.emit_struct(self.abi.structs[name]))
            else:
                continue
            lines.append("")

        for opaque in self.ordered_opaques():
            lines.extend(self.emit_opaque(opaque))
            lines.append("")

        for alias in self.ordered_aliases():
            lines.extend(self.emit_alias(alias))
            lines.append("")

        constants = self.emit_constants()
        if constants:
            lines.extend(constants)
            lines.append("")

        for func in self.ordered_functions():
            lines.extend(self.emit_function(func))

        lines.extend(self.epilogue())
        lines.extend(self.assertions())
        lines.extend(["", "#endif /* " + self.include_guard + " */"])
        return lines

    def banner_block(self) -> list[str]:
        if self.own_banner is not None:
            return list(self.own_banner)
        return [
            "/* Bindings for " + self.label + ", generated from the C header of the Louder engine.",
            " *",
            " * Do not edit. Run `python engine/bindings/generate.py` and this file comes back.",
            " */",
        ]

    def assertions(self) -> list[str]:
        return []


def literal(type: Type, value: int | float) -> str:
    """A constant in a form C will accept as an expression."""
    if isinstance(type, Scalar) and type.name == "float":
        return repr(float(value)) + "f"
    if isinstance(value, float):
        return repr(value)
    if isinstance(type, Scalar) and type.name in ("int64_t", "uint64_t", "long long", "size_t"):
        return str(value) + ("uLL" if value >= 0 else "LL")
    return str(value)