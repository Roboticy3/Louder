"""The interface a language plugs into.

Most FFI languages want the same four things from a header: a type per struct,
a type per enum, a declaration per function, and a name per constant. That is
most of what a backend has to say, so `Backend` writes those from the model and
leaves a backend to override only where its language differs — how a pointer is
spelled, whether a field name is allowed, what a function declaration looks
like before it is called.

A backend that only needs those overrides should be a hundred lines. A backend
that also wants a convenience layer on top overrides `runtime`, which is handed
the whole model and returns whatever extra files it likes.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from ..ir import (
    Abi,
    Alias,
    Array,
    Const,
    Enum,
    Field,
    Func,
    Layout,
    Named,
    Opaque,
    Pointer,
    Scalar,
    Struct,
    TaggedUnion,
    Target,
    Type,
    Variant,
    VOID,
)
from ..naming import Names, doc_lines

TIERS = ("deep", "solid", "broad", "long")


@dataclass
class Options:
    """Everything the generator was asked to do, in one place."""

    out: Path
    engine: Path
    library: Optional[Path] = None
    wasm: Optional[Path] = None
    header_name: str = "louder_rust.h"
    package: str = "louder"
    languages: tuple[str, ...] = ()
    indent: str = "  "


@dataclass
class Emitted:
    """What a backend produced: paths relative to its own directory, and text.

    `binaries` is for the one thing a generator does not generate: an artefact
    the language has to load at runtime, which was built by the engine's own
    toolchain and only copied here. Keeping it separate from `files` means the
    `--check` mode can compare text without reading a megabyte of wasm.
    """

    files: dict[str, str] = field(default_factory=dict)
    binaries: dict[str, bytes] = field(default_factory=dict)

    def add(self, path: str, text: str) -> None:
        self.files[path.replace("\\", "/")] = text

    def add_binary(self, path: str, data: bytes) -> None:
        self.binaries[path.replace("\\", "/")] = data

    def add_lines(self, path: str, lines: list[str], *, final_newline: bool = True) -> None:
        self.add(path, "\n".join(lines) + ("\n" if final_newline else ""))


class Backend:
    """The base every language backend extends."""

    # What to call this language, and where it sits in the ordering of effort.
    name: str = ""
    label: str = ""
    tier: str = "broad"
    # How deep the convenience layer goes. A backend at `deep` gets the whole
    # ergonomic treatment: names as strings, buffers that size themselves,
    # variants decoded, iteration written as a loop.
    convenience: str = "none"
    # Set by backends that can only be written against a built artefact.
    needs_wasm: bool = False
    needs_library: bool = False

    def __init__(self, abi: Abi, layout: Layout, options: Options):
        self.abi = abi
        self.layout = layout
        self.options = options
        self.names = self.make_names()

    # -- setup ------------------------------------------------------------

    def make_names(self) -> Names:
        return Names(self.name)

    @property
    def target(self) -> Target:
        return self.layout.target

    # -- entry point ------------------------------------------------------

    def generate(self) -> Emitted:
        out = Emitted()
        self.build(out)
        return out

    def build(self, out: Emitted) -> None:
        """Write every file. Overridden by anything with more than one file."""
        out.add_lines(self.primary_filename(), self.render())

    def primary_filename(self) -> str:
        return f"louder.{self.extension}"

    extension: str = "txt"

    # -- the parts a backend usually overrides ----------------------------

    def banner(self) -> list[str]:
        """The comment every generated file opens with.

        The second line matters more than the first: a reader who lands in one
        of these directories needs to know that editing the file is wasted work
        before they start.
        """
        prefix = self.comment_prefix()
        return [
            prefix + " Bindings for " + self.label + ", generated from the C header of the Louder engine.",
            prefix,
            prefix + " Do not edit. Run `python engine/bindings/generate.py` and this file comes back.",
        ]

    def runtime(self, relative: str) -> Path:
        """A file that does not depend on the ABI, copied out as it is.

        Some languages need a little code that is about the language's memory
        model rather than about this engine. That is written once here rather
        than generated, because generating it would mean embedding a source
        file inside a string, which makes it unreadable and unlintable.
        """
        path = Path(__file__).resolve().parent.parent / "runtime" / relative
        if not path.is_file():
            raise FileNotFoundError("no runtime file at " + str(path))
        return path

    def type_expr(self, type: Type, *, deref: bool = False) -> str:
        raise NotImplementedError

    def render(self) -> list[str]:
        """The default single-file rendering: types, constants, then functions."""
        lines: list[str] = []
        lines.extend(self.banner())
        lines.append("")

        lines.extend(self.preamble())

        for enum in self.ordered_enums():
            lines.extend(self.emit_enum(enum))
            lines.append("")

        # One ordering for every aggregate, so a struct that holds a union is
        # written after it rather than after whichever list happened to come
        # first.
        for name in self.declaration_order():
            union = self.abi.unions.get(name)
            if union is not None:
                lines.extend(self.emit_union(union))
                lines.append("")
                continue
            struct = self.abi.structs.get(name)
            if struct is None:
                continue
            emitter = self.emit_union_body if name in self.abi.union_bodies else self.emit_struct
            lines.extend(emitter(struct))
            lines.append("")

        for opaque in self.ordered_opaques():
            lines.extend(self.emit_opaque(opaque))
            lines.append("")

        for alias in self.ordered_aliases():
            lines.extend(self.emit_alias(alias))
            lines.append("")

        consts = self.emit_constants()
        if consts:
            lines.extend(consts)
            lines.append("")

        for func in self.ordered_functions():
            lines.extend(self.emit_function(func))

        lines.extend(self.epilogue())
        return lines

    # -- ordering ---------------------------------------------------------

    def ordered_enums(self) -> list[Enum]:
        return [self.abi.enums[name] for name in sorted(self.abi.enums)]

    def ordered_structs(self) -> list[Struct]:
        """Every struct, with everything it contains already declared.

        A union and a union body count as declarations here even though they are
        emitted separately: `ConditionOutcome` holds a `LouderVariant`, and a
        language that wants `ConditionOutcome` first needs the union to exist.
        """
        return [self.abi.structs[name] for name in self.declaration_order() if name in self.abi.structs]

    def declaration_order(self) -> list[str]:
        """Aggregate names, dependencies first.

        cbindgen writes the header in an order C accepts, but not necessarily
        one that `ctypes`, Go or C# accept, all of which need a type defined
        before anything that holds it. The walk below is that order, found from
        the declarations themselves.
        """
        if getattr(self, "_order", None) is None:
            self._order = self._walk_order()
        return self._order

    _order: Optional[list[str]] = None

    def _walk_order(self) -> list[str]:
        order: list[str] = []
        state: dict[str, int] = {}  # 1 = visiting, 2 = done

        def visit(name: str) -> None:
            if state.get(name) == 2:
                return
            if state.get(name) == 1:
                # A cycle would need a struct holding itself by value, which C
                # does not allow and neither does the engine.
                return
            state[name] = 1
            for dependency in self.dependencies_of(name):
                visit(dependency)
            state[name] = 2
            order.append(name)

        everything = list(self.abi.structs) + list(self.abi.unions) + list(self.abi.enums)
        for name in sorted(everything):
            visit(name)
        return order

    def dependencies_of(self, name: str) -> list[str]:
        """The declarations that have to come before this one."""
        wanted: list[str] = []

        def add(type: Type) -> None:
            base = self.underlying_name(type)
            if base and base != name and base not in wanted:
                wanted.append(base)

        struct = self.abi.structs.get(name)
        if struct is not None:
            for field in struct.fields:
                add(field.type)
            return wanted

        union = self.abi.unions.get(name)
        if union is not None:
            add(union.tag_type)
            body = self.abi.structs.get(name + "_Body")
            if body is not None:
                add(Named(name + "_Body"))
            return wanted

        enum = self.abi.enums.get(name)
        if enum is not None:
            add(enum.underlying)
        return wanted

    def underlying_name(self, type: Type) -> str | None:
        """The declared name a type refers to, looking through pointers."""
        seen = 0
        while isinstance(type, Pointer):
            type = type.to
            seen += 1
            if seen > 32:
                return None
        if not isinstance(type, Named):
            return None
        return type.name

    def aggregate_names(self) -> list[str]:
        """Every aggregate declaration, in the order they can be written."""
        return [name for name in self.declaration_order() if name in self.abi.structs or name in self.abi.unions]

    def ordered_union_bodies(self) -> list[str]:
        return sorted(self.abi.union_bodies)

    def ordered_unions(self) -> list[TaggedUnion]:
        return [self.abi.unions[name] for name in sorted(self.abi.unions)]

    def ordered_opaques(self) -> list[Opaque]:
        return [self.abi.opaques[name] for name in sorted(self.abi.opaques)]

    def ordered_aliases(self) -> list[Alias]:
        return [self.abi.aliases[name] for name in sorted(self.abi.aliases)]

    def ordered_functions(self) -> list[Func]:
        return list(self.abi.funcs)

    def emit_constants(self) -> list[str]:
        return []

    # -- hooks with a default --------------------------------------------

    def preamble(self) -> list[str]:
        return []

    def epilogue(self) -> list[str]:
        return []

    def emit_enum(self, enum: Enum) -> list[str]:
        lines = doc_lines(enum.doc, self.comment_prefix())
        for variant, value in enum.variants:
            lines.append(f"{self.comment_prefix()} {variant} = {value if value is not None else ''}".rstrip())
        if lines:
            lines.append("")
        lines.append(f"{self.typedef_start()}{self.names.type(enum.name)};")
        return lines

    def emit_struct(self, struct: Struct) -> list[str]:
        name = self.names.type(struct.name)
        lines = [f"{self.typedef_start()}{name} {{"]
        for placed in self.layout.fields(struct):
            lines.append(f"{self.indent}{self.type_expr(placed.type)} {self.names.field(placed.name)};")
        lines.append("};")
        return lines

    def emit_union_body(self, struct: Struct) -> list[str]:
        name = self.names.type(struct.name)
        lines = [f"{self.typedef_start()}{name} {{"]
        for placed in self.layout.fields(struct):
            lines.append(f"{self.indent}{self.type_expr(placed.type)} {self.names.field(placed.name)};")
        lines.append("};")
        return lines

    def emit_union(self, union: TaggedUnion) -> list[str]:
        name = self.names.type(union.name)
        body = self.names.type(union.name + "_Body")
        lines = [f"{self.typedef_start()}{name} {{", f"{self.indent}{self.type_expr(union.tag_type)} {self.names.field('tag')};", f"{self.indent}{body} {self.names.field('body')};", "};"]
        return lines

    def emit_opaque(self, opaque: Opaque) -> list[str]:
        # C spells an opaque handle as a bare tag; Rust has no such thing and
        # spells it as an empty struct, which is never looked inside anyway.
        return [
            self.doc_block(opaque.doc, "The " + opaque.name + " handle."),
            "#[repr(C)]",
            "#[derive(Debug)]",
            "pub struct " + self.names.type(opaque.name) + " {",
            "    _unused: [u8; 0],",
            "}",
        ]

    def emit_alias(self, alias: Alias) -> list[str]:
        return [f"{self.typedef_start()}{self.names.type(alias.name)} {self.type_expr(alias.target)};"]

    def emit_function(self, func: Func) -> list[str]:
        args = self.call_arguments(func)
        lines = [f"{self.type_expr(func.ret)} {self.names.func(func.name)}({', '.join(args)});"]
        lines.extend(doc_lines(func.doc, self.comment_prefix()))
        return lines

    def call_arguments(self, func: Func) -> list[str]:
        out: list[str] = []
        for index, param in enumerate(func.params):
            name = self.names.arg(param.name, index)
            out.append(f"{self.type_expr(param.type)} {name}")
        return out

    # -- shared helpers ---------------------------------------------------

    def comment_prefix(self) -> str:
        return "//"

    def typedef_start(self) -> str:
        return "typedef"

    @property
    def indent(self) -> str:
        return self.options.indent

    def doc(self, text: str) -> list[str]:
        return doc_lines(text, self.comment_prefix())

    def doc_line(self, text: str, fallback: str = "") -> str:
        """One line of comment, terminated.

        A block comment that is not closed swallows everything after it, and the
        symptom shows up as a missing export rather than as a syntax error, so
        this is worth having in one place rather than spelled out each time.
        """
        body = " ".join((text or fallback or "").split())
        prefix = self.comment_prefix()
        if prefix == "//":
            return prefix + " " + (body or fallback)
        if len(body) + len(prefix) + 6 <= 100:
            return prefix + " " + body + " */"
        return prefix + " * " + body[:88] + "... */"

    def doc_block(self, text: str, fallback: str = "") -> str:
        """A one-paragraph comment, opened and closed, or several lines long."""
        body = (text or "").strip()
        prefix = self.comment_prefix()
        if prefix == "//":
            return "\n".join(prefix + " " + line for line in doc_lines(body, prefix + " "))
        if not body:
            body = fallback
        if len(" ".join(body.split())) + len(prefix) + 6 <= 100:
            return prefix + " " + " ".join(body.split()) + " */"
        lines = [prefix]
        for paragraph in body.split("\n\n"):
            lines.append(prefix + " * " + " ".join(paragraph.split()))
            lines.append(prefix + " *")
        if lines[-1] == prefix + " *":
            lines.pop()
        lines.append(prefix + " */")
        return "\n".join(lines)

    def is_bitflag(self, name: str) -> bool:
        return name in self.abi.bitflags

    # -- a note for every backend about the shapes it will meet ------------

    def shapes(self) -> dict[str, object]:
        return {
            "structs": len(self.abi.structs),
            "unions": len(self.abi.unions),
            "enums": len(self.abi.enums),
            "opaques": len(self.abi.opaques),
            "aliases": len(self.abi.aliases),
            "constants": len(self.abi.consts),
            "functions": len(self.abi.funcs),
            "bitflags": sorted(self.abi.bitflags),
        }
