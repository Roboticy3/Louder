"""Python bindings, in two layers.

`_ffi` is the boundary exactly as the header describes it: `ctypes` structures,
the constants, and the entry points with their signatures attached. It is what
everything else is built on, and it is the only module that knows the engine is
written in Rust.

Everything above it is written for the person using the engine rather than for
the person calling C. Strings cross the boundary as bytes and come back as
`bytes`, so names are taken and returned as `str`; buffers are the other half of
it, so a call that fills one passes a `bytearray` and gets the bytes written
back; tagged unions are switched on rather than read at an offset, so a variant
is `int | float | str` and an event carries a kind and a value; and the `next_*`
family, which is a cursor in a loop in every language without generators,
becomes an actual iterator.

Nothing here is written per function. The wrappers are derived from the
signatures in the header, so an entry point added in Rust arrives in Python
without anyone touching this file.
"""

from __future__ import annotations

from ..ir import Enum, Named, Pointer, Scalar, Type
from ..naming import Names
from .base import Backend, Emitted

_CTYPES = {
    "bool": "c_bool",
    "int8_t": "c_int8",
    "uint8_t": "c_uint8",
    "int16_t": "c_int16",
    "uint16_t": "c_uint16",
    "int32_t": "c_int32",
    "uint32_t": "c_uint32",
    "int64_t": "c_int64",
    "uint64_t": "c_uint64",
    "float": "c_float",
    "double": "c_double",
    "size_t": "c_size_t",
    "ptrdiff_t": "c_ssize_t",
}

# Structs the convenience layer writes out by hand, because it changes what is
# inside them on the way through.
RECORD_OVERRIDES = frozenset({"LouderEvent", "LouderAsk"})


class PythonBackend(Backend):
    name = "python"
    label = "Python"
    tier = "deep"
    convenience = "full"
    extension = "py"
    needs_library = True

    def make_names(self) -> Names:
        # The header's spelling is already `snake_case` and already reads well in
        # Python, including the `_Tag` and `_Body` suffixes that say what those
        # types are for. Recasing would only hide that.
        return Names("python", type_case=str, func_case=str, field_case=str, const_case=str)

    def comment_prefix(self) -> str:
        return "#"

    # -- files ------------------------------------------------------------

    def build(self, out: Emitted) -> None:
        out.add_lines("louder/_ffi.py", self.render_ffi())
        out.add_lines("louder/__init__.py", self.render_api())
        out.add("louder/py.typed", "")
        out.add_lines("README.md", self.render_readme())
        out.add_lines("pyproject.toml", self.render_pyproject())

    # ------------------------------------------------------------------
    # The raw layer
    # ------------------------------------------------------------------

    def ctypes_expr(self, type: Type) -> str:
        if isinstance(type, Pointer):
            return "POINTER(" + self.ctypes_expr(type.to) + ")"
        if isinstance(type, Scalar):
            if type.name == "void":
                return "None"
            return _CTYPES.get(type.name, self.names.type(type.name))
        if isinstance(type, Named):
            alias = self.abi.aliases.get(type.name)
            if alias is not None:
                return self.ctypes_expr(alias.target)
            return self.names.type(type.name)
        raise KeyError("no ctypes spelling for " + repr(type))

    def render_ffi(self) -> list[str]:
        lines: list[str] = []
        lines.extend(self.banner())
        lines.append('"""')
        lines.append("The C boundary, exactly as the header describes it.")
        lines.append("")
        lines.append("Nothing here interprets anything. A name is an integer, a tagged")
        lines.append("union is a tag and a block of bytes, and a string is a length and a")
        lines.append("buffer. The convenience layer above is where those become names,")
        lines.append("variants and text; use it unless you are writing one yourself.")
        lines.append('"""')
        lines.append("")
        lines.append("from __future__ import annotations")
        lines.append("")
        lines.append("import ctypes")
        lines.append("import os")
        lines.append("")
        lines.append("from ctypes import POINTER")
        lines.append("from ctypes import c_bool, c_double, c_float, c_int8, c_int16, c_int32, c_int64")
        lines.append("from ctypes import c_size_t, c_ssize_t, c_uint8, c_uint16, c_uint32, c_uint64")
        lines.append("from typing import Any")
        lines.append("")
        lines.append('__all__ = ["LIBRARY_NAMES", "load_library", "available"]')
        lines.append("")
        lines.append("")
        lines.extend(self.ffi_loader())
        lines.append("")
        lines.extend(self.ffi_types())
        lines.append("")
        lines.extend(self.ffi_constants())
        lines.append("")
        lines.extend(self.ffi_functions())
        return lines

    def ffi_loader(self) -> list[str]:
        names = ["louder_rust.dll", "liblouder_rust.so", "liblouder_rust.dylib", "louder_rust.so", "louder_rust.dylib"]
        listed = ", ".join('"' + n + '"' for n in names)
        return [
            "LIBRARY_NAMES = (" + listed + ",)",
            "",
            "# Point this at the library to skip the search.",
            'LIBRARY_ENV = "LOUDER_LIBRARY"',
            "",
            "_library: ctypes.CDLL | None = None",
            "",
            "",
            "def library_candidates() -> list[str]:",
            '    """Where to look for the engine, in the order to try it."""',
            "    override = os.environ.get(LIBRARY_ENV)",
            "    if override:",
            "        return [override]",
            "",
            '    here = os.path.dirname(os.path.abspath(__file__))',
            '    roots = [os.path.normpath(os.path.join(here, "..", "..")), here]',
            "    out: list[str] = []",
            "    for root in roots:",
            "        for name in LIBRARY_NAMES:",
            "            out.append(os.path.join(root, name))",
            "    return out",
            "",
            "",
            "def load_library(path: str | None = None) -> ctypes.CDLL:",
            '    """Load the engine, or raise with every place that was tried."""',
            "    global _library",
            "    if path is None and _library is not None:",
            "        return _library",
            "",
            "    tried = [path] if path else library_candidates()",
            "    errors: list[str] = []",
            "    for candidate in tried:",
            "        try:",
            "            found = ctypes.CDLL(candidate)",
            "        except OSError as error:",
            '            errors.append("  " + str(candidate) + ": " + str(error))',
            "            continue",
            "        _bind(found)",
            "        if path is None:",
            "            _library = found",
            "        return found",
            "",
            "    raise OSError(",
            '        "the Louder engine could not be loaded. Build it with "',
            '        + "cargo build -p louder_rust, then point " + LIBRARY_ENV + " at the "',
            '        + "library or pass a path. Tried:" + chr(10) + chr(10).join(errors)',
            "    )",
        ]

    def ffi_types(self) -> list[str]:
        lines = ["# --- types " + "-" * 58, ""]
        for enum in self.ordered_enums():
            lines.append(self.names.type(enum.name) + " = " + self.ctypes_expr(enum.underlying))
            for variant, value in enum.variants:
                lines.append(self.names.type(enum.name) + "_" + variant + " = " + str(value if value is not None else 0))
        lines.append("")

        # One walk over every aggregate, so that a struct holding a union is
        # declared after it. Splitting structs and unions into two lists and
        # hoping for the best is how `ConditionOutcome` ends up referring to a
        # `LouderVariant` that has not been defined yet.
        for name in self.declaration_order():
            if name in self.abi.unions:
                union = self.abi.unions[name]
                lines.append("")
                lines.extend(self.docstring(union.doc, top=True))
                lines.append("class " + self.names.type(name) + "(ctypes.Structure):")
                lines.append("    _fields_ = [")
                lines.append("        ('tag', " + self.ctypes_expr(union.tag_type) + "),")
                lines.append("        ('body', " + self.names.type(name + "_Body") + "),")
                lines.append("    ]")
                lines.append("    _anonymous_ = ('body',)")
                continue
            struct = self.abi.structs.get(name)
            if struct is None:
                continue
            overlapping = name in self.abi.union_bodies
            base = "ctypes.Union" if overlapping else "ctypes.Structure"
            lines.append("")
            lines.extend(self.docstring(struct.doc, top=True))
            lines.append("class " + self.names.type(name) + "(" + base + "):")
            lines.append("    _fields_ = [")
            for placed in self.layout.fields(struct):
                lines.append("        (" + repr(placed.name) + ", " + self.ctypes_expr(placed.type) + "),")
            lines.append("    ]")
        lines.append("")

        for opaque in self.ordered_opaques():
            lines.append("")
            lines.append("class " + self.names.type(opaque.name) + "(ctypes.Structure):")
            lines.append("    " + self.one_line_doc(opaque.doc, opaque.name))
            lines.append("    pass")
        lines.append("")

        for alias in self.ordered_aliases():
            lines.append(self.names.type(alias.name) + " = " + self.ctypes_expr(alias.target))
        lines.append("")
        return lines

    def ffi_constants(self) -> list[str]:
        lines = ["", "# --- constants " + "-" * 54, ""]
        for const in self.abi.consts:
            if isinstance(const.type, Named):
                value = self.ctypes_expr(const.type) + ".bits"
            else:
                value = repr(const.value)
            lines.append(const.name + " = " + value)
        return lines

    def ffi_functions(self) -> list[str]:
        lines = [
            "",
            "# --- entry points " + "-" * 51,
            "",
            "# Each name is bound the first time it is used, so importing the types",
            "# above does not require the engine to be present at all.",
            "_PROTOTYPES: dict[str, tuple[Any, tuple[Any, ...]]] = {",
        ]
        for func in self.ordered_functions():
            ret = self.ctypes_expr(func.ret)
            args = [self.ctypes_expr(p.type) for p in func.params]
            lines.append("    " + repr(func.name) + ": (" + ret + ", (" + "".join(a + ", " for a in args) + ")),")
        lines.append("}")
        lines.append("")
        lines.extend(
            [
                "def _bind(found: ctypes.CDLL) -> None:",
                "    for name, (restype, argtypes) in _PROTOTYPES.items():",
                "        entry = getattr(found, name)",
                "        entry.restype = restype",
                "        entry.argtypes = argtypes",
                "        globals()[name] = entry",
                "",
                "",
                "def __getattr__(name: str) -> Any:",
                '    """Bind an entry point the first time it is asked for."""',
                "    if name in _PROTOTYPES:",
                "        entry = getattr(load_library(), name)",
                "        restype, argtypes = _PROTOTYPES[name]",
                "        entry.restype = restype",
                "        entry.argtypes = argtypes",
                "        globals()[name] = entry",
                "        return entry",
                "    raise AttributeError(",
                "        'module ' + __name__ + ' has no attribute ' + repr(name)",
                "    )",
                "",
                "",
                "def available() -> bool:",
                '    """Whether the engine can be loaded at all."""',
                "    try:",
                "        load_library()",
                "    except OSError:",
                "        return False",
                "    return True",
                "",
            ]
        )
        return lines

    # ------------------------------------------------------------------
    # The convenience layer
    # ------------------------------------------------------------------

    def render_api(self) -> list[str]:
        lines: list[str] = []
        lines.extend(self.banner())
        lines.extend(
            [
                '"""',
                "The Louder engine, from Python.",
                "",
                "A `Louder` owns one engine context and everything that comes with it:",
                "the name table, every conversation, and the event queue. It is released",
                "when it is closed, or when it goes out of scope as a context manager.",
                "",
                "    with Louder() as engine:",
                "        engine.load_config(document)",
                "        engine.load_conversation('chat', document)",
                "        actor = engine.add_actor('chat', 0, speak_as=['A'])",
                "        engine.choose('chat', actor, 0)",
                "        for event in engine.drain_events():",
                "            print(event.kind, event.value)",
                "",
                "Strings cross the boundary as bytes and are decoded here as UTF-8, which",
                "is the encoding the engine reads JSON in. Anything else is handed over",
                "untouched, as `bytes`.",
                '"""',
                "",
                "from __future__ import annotations",
                "",
                "import ctypes",
                "",
                "from dataclasses import dataclass",
                "from enum import IntEnum, IntFlag",
                "from typing import Any, Iterator, Sequence",
                "",
                "from . import _ffi",
                "from ._ffi import (",
            ]
        )
        for const in self.abi.consts:
            if isinstance(const.type, Scalar):
                lines.append("    " + const.name + ",")
        lines.append(")")
        lines.append("")
        lines.append("__all__ = [")
        for entry in [
            "Louder",
            "LouderFailure",
            "LouderError",
            "LoadError",
            "Event",
            "Ask",
            "Motion",
            "ChoiceAppearance",
            "AdditionalDataChange",
            "ConditionOutcome",
            "LoadFailure",
            "NAME_NONE",
            "ACTOR_NONE",
            "BLANK_POSITION",
        ] + self.enum_names():
            lines.append("    " + repr(entry) + ",")
        lines.append("]")
        lines.append("")
        lines.append("")
        lines.extend(self.api_enums())
        lines.append("")
        lines.extend(self.api_dataclasses())
        lines.append("")
        lines.extend(self.api_exceptions())
        lines.append("")
        lines.extend(self.api_kinds())
        lines.append("")
        lines.extend(self.API_HELPERS.strip("\n").splitlines())
        lines.append("")
        lines.extend(self.render_class())
        return lines

    def enum_names(self) -> list[str]:
        names = [self.names.type(name) for name in sorted(self.abi.enums)]
        names += [self.names.type(name) for name in sorted(self.abi.bitflags)]
        return names

    def api_enums(self) -> list[str]:
        lines = ["# --- vocabularies " + "-" * 49, ""]

        for name in sorted(self.abi.enums):
            enum = self.abi.enums[name]
            lines.append("class " + self.names.type(name) + "(IntEnum):")
            lines.extend(self.docstring(enum.doc, indent="    "))
            lines.extend(self.enum_members(enum))
            lines.append("")
            lines.append("")

        for name in sorted(self.abi.bitflags):
            flags = self.abi.bitflags[name]
            lines.append("class " + self.names.type(name) + "(IntFlag):")
            lines.extend(self.docstring(flags.doc, indent="    "))
            for variant, value in flags.variants:
                lines.append("    " + self.member_name(variant) + " = " + str(value))
            lines.append("")
            lines.append("")

        # An integer alias stands in for a Rust `type` rather than an `enum`, so
        # cbindgen writes a typedef and the constants beside it, and nothing in
        # the header says which constants belong to which alias. The rule below
        # puts them back together: a constant family is an alias's codes when
        # every member starts with a word the alias's name also spells out.
        for alias, prefix, family in self.code_families():
            lines.append("class " + self.names.type(alias.name) + "(IntEnum):")
            lines.extend(self.docstring(self.code_doc(alias), indent="    "))
            for const in family:
                member = self.member_name(title_words(const.name[len(prefix) :]))
                lines.append("    " + member + " = " + str(int(const.value)))
            lines.append("")
            lines.append("")

        for const in self.abi.consts:
            if isinstance(const.type, Scalar):
                lines.append(const.name + " = " + str(int(const.value)))
        return lines

    def enum_members(self, enum: Enum) -> list[str]:
        lines: list[str] = []
        taken: set[int] = set()
        running = 0
        for variant, explicit in enum.variants:
            running = explicit if explicit is not None else running
            while running in taken:
                running += 1
            taken.add(running)
            lines.append("    " + self.member_name(variant) + " = " + str(running))
        return lines

    def member_name(self, variant: str) -> str:
        """An enum member, in the one spelling Python allows."""
        if variant in self.names.reserved:
            return variant.upper() + "_"
        return variant

    def code_families(self) -> list[tuple[object, str, list]]:
        """Pair each integer alias with the constants that are its codes.

        `cbindgen` writes a Rust `pub type LoadError = i32` as a typedef and its
        codes as `LOUDER_LOAD_OK` and friends, with nothing joining the two up.
        The join available here is the naming: a constant is one of an alias's
        codes when the first word after the prefix shared by all constants
        also appears in the alias's own name, spelled however each side likes.
        """
        import os as _os

        scalars = [c for c in self.abi.consts if isinstance(c.type, Scalar)]
        if len(scalars) < 2:
            return []
        shared = _os.path.commonprefix([c.name for c in scalars])
        if not shared:
            return []

        families: list[tuple[object, str, list]] = []
        for name in sorted(self.abi.aliases):
            alias = self.abi.aliases[name]
            if not isinstance(self.abi.resolve(alias.target), Scalar):
                continue
            letters = set(ch.lower() for ch in name if ch.isalpha())
            head_prefix = shared
            family = []
            for const in scalars:
                rest = const.name[len(shared) :]
                head = rest.split("_")[0].lower()
                if head and all(ch in letters for ch in head):
                    family.append((const, rest))
            if len(family) < 2:
                continue
            family_prefix = _os.path.commonprefix([rest for _c, rest in family])
            ordered = sorted((c for c, _r in family), key=lambda c: int(c.value))
            families.append((alias, head_prefix + family_prefix, ordered))
        return families

    def code_doc(self, alias) -> str:
        return "Why a document was refused. Zero means it was not refused."

    def code_names(self) -> list[str]:
        return [self.names.type(alias.name) for alias, _prefix, _family in self.code_families()]

    def api_dataclasses(self) -> list[str]:
        lines = ["# --- records " + "-" * 53, ""]
        for name in sorted(self.abi.structs):
            if name in self.abi.union_bodies or name in RECORD_OVERRIDES:
                continue
            struct = self.abi.structs[name]
            lines.append("@dataclass")
            lines.append("class " + self.names.type(name) + ":")
            lines.extend(self.docstring(struct.doc, indent="    "))
            for placed in self.layout.fields(struct):
                lines.append("    " + placed.name + ": " + self.python_type(placed.type))
            lines.append("")
            lines.append("")
        return lines

    def python_type(self, type: Type) -> str:
        if isinstance(type, Pointer):
            return "Any"
        if isinstance(type, Scalar):
            return "float" if type.name in ("float", "double") else "int"
        alias = self.abi.aliases.get(type.name) if isinstance(type, Named) else None
        if alias is not None:
            return self.python_type(alias.target)
        if isinstance(type, Named):
            if type.name in self.abi.unions:
                # A tagged union is read by switching on its tag, which the layer
                # above does; no single type describes it.
                return "Any"
            if type.name in self.abi.structs:
                return self.names.type(type.name)
        return "int"

    def api_exceptions(self) -> list[str]:
        return [
            "# --- failures " + "-" * 52,
            "",
            "",
            "class LouderFailure(Exception):",
            '    """Something went wrong that the caller has to know about."""',
            "",
            "",
            "class LoadFailed(LouderFailure):",
            '    """A document the engine refused, and why."""',
            "",
            "    def __init__(self, code: int) -> None:",
            "        self.code = LoadError(code)",
            "        super().__init__(self.code.name)",
            "",
        ]

    def api_kinds(self) -> list[str]:
        """Tag value to arm name, for each tagged union.

        Two names exist for one arm. The tag enum carries the name the engine
        gave it, `RegularMotion`; the body field carries the name cbindgen gave
        it, `regular_motion`, which is how the raw struct is read. A caller
        wants the first, so it is looked up in the tag rather than guessed at
        from the body.

        It also matters that the lookup is by value and not by position: the
        tag is an `int` that came back from the engine, and reading it as an
        index into a list is how an engine that grew a variant silently reports
        the wrong one.
        """
        lines = ["# --- kinds " + "-" * 55, ""]
        for name in sorted(self.abi.unions):
            union = self.abi.unions[name]
            if not self.abi.union_arms(union):
                continue
            lines.append(self.names.type(name) + "_KINDS: dict[int, str] = {")
            for value, variant, _body in self.abi.union_arms(union):
                lines.append("    " + str(value) + ": " + repr(variant) + ",")
            lines.append("}")
            lines.append("")
        lines.append("")
        return lines

    def docstring(self, doc: str, indent: str = "", *, top: bool = False) -> list[str]:
        """A docstring, or the comment standing in for one above a class.

        A class body has to start with its docstring, so a class that follows a
        comment cannot use one. The engine's description of `Motion` belongs on
        `Motion` either way, and a `#` comment above the class reads the same.
        """
        text = (doc or "").strip()
        prefix = "# " if top else ""
        pad = "" if top else indent
        if not text:
            return [prefix.rstrip()]
        paragraphs = text.split("\n\n")
        single = " ".join(paragraphs[0].split())
        if len(paragraphs) == 1 and len(single) + len(pad) + len(prefix) + 6 <= 88:
            return [pad + prefix + '"""' + single + '"""']
        lines = [pad + prefix + '"""']
        for paragraph in paragraphs:
            lines.append(pad + prefix + " ".join(paragraph.split()))
            lines.append("")
        if lines[-1] == "":
            lines.pop()
        lines.append(pad + prefix + '"""')
        return lines

    def one_line_doc(self, doc: str, fallback: str) -> str:
        text = " ".join((doc or "").split())
        if len(text) + 8 <= 88:
            return '"""' + (text or fallback) + '"""'
        return '"""' + (text or fallback)[:80] + '..."""'

    # ------------------------------------------------------------------
    # Iteration
    # ------------------------------------------------------------------

    # Arguments the engine names after things that are names. They are taken as
    # text by the caller and declared here, which is what makes `hubs(c, "main")`
    # read the way the engine's own documentation does.
    NAME_ARGUMENTS = frozenset({"conversation", "channel", "speaker", "key", "state", "text", "ask"})

    # What each cursor ends on. It is the value the header documents as the end
    # of the walk, and it is the only thing that tells a loop when to stop.
    SENTINELS = {
        "LouderName": "NAME_NONE",
        "LouderActor": "ACTOR_NONE",
        "LouderPosition": "BLANK_POSITION",
    }

    def iterators(self) -> list[Func]:
        """The `next_*` functions, which are cursors written out by hand.

        Every one of them has the same shape: a `start`, the thing it returns,
        and a sentinel to say the walk is over. That is a loop in any language
        without generators, and a `while True` here.
        """
        found = []
        for func in self.abi.funcs:
            if not func.name.startswith("next_"):
                continue
            start = [p for p in func.params if p.name == "start"]
            if not start or not isinstance(start[0].type, Named):
                continue
            # A cursor returns the same kind of thing it was last handed.
            if not isinstance(func.ret, Named) or func.ret.name != start[0].type.name:
                continue
            if func.ret.name not in self.SENTINELS:
                continue
            found.append(func)
        return found

    def section_iterators(self) -> list[str]:
        out = [
            "    # --- iteration " + "-" * 51,
            "",
            "    # Every one of these is a cursor in the C header: a `start`, the thing",
            "    # it returns, and a sentinel for the end of the walk. Here they are",
            "    # generators, so the walk reads as a `for` and the caller cannot forget",
            "    # to advance.",
            "",
        ]
        for func in self.iterators():
            end = self.SENTINELS[func.ret.name]
            args = [p for p in func.params if p.name not in ("start", "ctx")]
            signature = ", ".join(self.arg_signature(p, index) for index, p in enumerate(args))
            out.append("    def " + self.iterator_name(func.name) + "(self" + (", " + signature if signature else "") + ") -> Iterator[int]:")
            out.extend(self.docstring(func.doc, indent="        "))
            call = ", ".join(["self._handle"] + [self.call_argument(p, index) for index, p in enumerate(args)])
            out.append("        start: Any = " + end)
            out.append("        while True:")
            out.append("            found = _ffi." + func.name + "(" + call + ", start)")
            out.append("            if found == " + end + ":")
            out.append("                return")
            out.append("            yield found")
            out.append("            start = found")
            out.append("")
        return out

    def arg_signature(self, param, index: int) -> str:
        name = self.names.arg(param.name, index)
        if isinstance(param.type, Named) and param.name in self.NAME_ARGUMENTS:
            return name + ": str | int"
        if isinstance(self.abi.resolve(param.type), Scalar) and param.name == "exact":
            return name + ": int = 0"
        return name + ": " + self.python_type(param.type)

    def call_argument(self, param, index: int) -> str:
        name = self.names.arg(param.name, index)
        if isinstance(param.type, Named) and param.name in self.NAME_ARGUMENTS:
            return "self.declare(" + name + ")"
        return name

    def iterator_name(self, func_name: str) -> str:
        return "iter_" + func_name[len("next_") :]

    # ------------------------------------------------------------------
    # The Louder class
    # ------------------------------------------------------------------

    def render_class(self) -> list[str]:
        lines: list[str] = []
        for section in (
            self.section_names(),
            self.section_context(),
            self.section_documents(),
            self.section_hubs(),
            self.section_actors(),
            self.section_state(),
            self.section_iterators(),
            self.section_events(),
            self.section_asks(),
            self.section_raw(),
        ):
            lines.extend(section)
            lines.append("")
        return lines

    def section_names(self) -> list[str]:
        return [
            "    # --- names " + "-" * 53,
            "",
            "    def lookup(self, text: str | bytes) -> int:",
            '        """The id of some text, or NAME_NONE if it was never declared."""',
            "        return _ffi.get_named_id(self._handle, *_with(self, text))",
            "",
            "    def declare(self, text: str | int) -> int:",
            '        """',
            "        The id of some text, declaring it if this is the first time.",
            "",
            "        An id is passed straight back, so a value that came out of one of",
            "        these calls can go into any other without being looked up again.",
            '        """',
            "        if isinstance(text, int):",
            "            return text",
            "        existing = self.lookup(text)",
            "        if existing != NAME_NONE:",
            "            return existing",
            "        return _ffi.declare_name(self._handle, *_with(self, text))",
            "",
            "    def name(self, text: str | int) -> int:",
            '        """An alias for declare, for when the text really is a name."""',
            "        return self.declare(text)",
            "",
            "    def text(self, name: int) -> str:",
            '        """The text behind an id, decoded as UTF-8."""',
            "        return _take(_ffi.read_name, self._handle, int(name)).decode('utf-8', 'replace')",
            "",
            "    def optional_text(self, name: int) -> str | None:",
            '        """The text behind an id, or None when the id names nothing."""',
            "        if int(name) == NAME_NONE:",
            "            return None",
            "        return self.text(name)",
            "",
        ]

    def section_context(self) -> list[str]:
        rules = self.bitflag_param("set_dialogue_rules", "rules")
        wrapped = "_bits(" + repr(rules) + ", rules)" if rules else "rules"
        return [
            "    # --- context " + "-" * 52,
            "",
            "    def rules(self) -> DialogueRules:",
            '        """The dialogue rules in effect."""',
            "        return DialogueRules(_ffi.get_dialogue_rules(self._handle).bits)",
            "",
            "    def set_rules(self, rules: int) -> None:",
            '        """Replace the dialogue rules. They take effect on the next move."""',
            "        _ffi.set_dialogue_rules(self._handle, " + wrapped + ")",
            "",
        ]

    def section_documents(self) -> list[str]:
        return [
            "    # --- documents " + "-" * 49,
            "",
            "    def load_conversation(self, conversation: str | int, document: str | bytes) -> LoadError:",
            '        """Load a conversation from a JSON document."""',
            "        name = self.declare(conversation)",
            "        return LoadError(_ffi.load_conversation_json(self._handle, name, *_with(self, document)))",
            "",
            "    def load_config(self, document: str | bytes) -> LoadError:",
            '        """Load the speaker config, which is a singleton."""',
            "        return LoadError(_ffi.load_conversation_config(self._handle, *_with(self, document)))",
            "",
            "    def clear_config(self) -> bool:",
            '        """Remove every speaker. Conversations are left alone."""',
            "        return _ffi.clear_config(self._handle) == 0",
            "",
            "    def remove_conversation(self, conversation: str | int) -> bool:",
            '        """Remove a conversation and the actors in it."""',
            "        return _ffi.remove_conversation(self._handle, self.declare(conversation)) == 0",
            "",
        ]

    def section_hubs(self) -> list[str]:
        out = ["    # --- hubs " + "-" * 54, ""]
        simple = [
            ("hub_text", "get_hub_text", "str", "position: int", "The text of a hub."),
            ("hub_speaker", "get_hub_speaker", "str", "position: int", "The speaker of a hub."),
            ("hub_channel", "get_hub_channel", "str", "position: int", "The channel of a hub."),
            ("hub_expressions", "get_hub_expressions", "str", "position: int", "The expressions of a hub."),
            ("hub_callbacks", "get_hub_callbacks", "str", "position: int", "The callbacks of a hub."),
            ("hub_priority", "get_hub_priority", "int", "position: int", "The priority of a hub."),
        ]
        for method, entry, result, args, summary in simple:
            out.append("    def " + method + "(self, conversation: str | int, " + args + ") -> " + result + ":")
            out.append('        """' + summary + '"""')
            out.append("        return self.text(_ffi." + entry + "(self._handle, self.declare(conversation), position))")
            out.append("")

        out.extend(
            [
                "    def hub_flags(self, conversation: str | int, position: int) -> HubFlags:",
                '        """The flags on a hub."""',
                "        return HubFlags(_ffi.get_hub_flags(self._handle, self.declare(conversation), position).bits)",
                "",
                "    def hub_additional(self, conversation: str | int, position: int, key: str | int) -> Any:",
                '        """Additional data on a hub, under a key."""',
                "        return self._variant(",
                "            _ffi.get_hub_additional(",
                "                self._handle, self.declare(conversation), position, self.declare(key)",
                "            )",
                "        )",
                "",
                "    def hub_choice_count(self, conversation: str | int, position: int, actor: int) -> int:",
                '        """How many choices are available to an actor on a hub."""',
                "        return _ffi.get_hub_choice_count(self._handle, self.declare(conversation), position, actor)",
                "",
                "    def hub_choice_text(self, conversation: str | int, position: int, actor: int, index: int) -> str:",
                '        """The text of one choice, which is the text of the hub it leads to."""',
                "        return self.text(",
                "            _ffi.get_hub_choice_text(self._handle, self.declare(conversation), position, actor, index)",
                "        )",
                "",
                "    def hub_choice_link(self, conversation: str | int, position: int, actor: int, index: int) -> int:",
                '        """Where one choice leads."""',
                "        return _ffi.get_hub_choice_link(self._handle, self.declare(conversation), position, actor, index)",
                "",
            ]
        )
        return out

    def bitflag_param(self, func_name: str, param_name: str) -> str | None:
        """The flags struct a parameter takes, if that is what it takes.

        Found from the declaration rather than named here, so a flags argument
        added in Rust is wrapped the same way as the ones already are.
        """
        if not self.abi.has_func(func_name):
            return None
        for param in self.abi.func(func_name).params:
            if param.name != param_name:
                continue
            if isinstance(param.type, Named) and param.type.name in self.abi.bitflags:
                return param.type.name
        return None

    def section_actors(self) -> list[str]:
        flags = self.bitflag_param("add_actor", "flags")
        wrapped = "_bits(" + repr(flags) + ", flags)" if flags else "flags"
        return [
            "    # --- actors " + "-" * 53,
            "",
            "    def add_actor(",
            "        self,",
            "        conversation: str | int,",
            "        position: int,",
            "        *,",
            "        speak_as: Sequence[str | int] = (),",
            "        listen_to: Sequence[str | int] = (),",
            "        flags: int = 0,",
            "    ) -> int:",
            '        """Put an actor into a conversation and return its id."""',
            "        return _ffi.add_actor(",
            "            self._handle,",
            "            self.declare(conversation),",
            "            position,",
            "            " + wrapped + ",",
            "            _name_array([self.declare(s) for s in speak_as]),",
            "            len(speak_as),",
            "            _name_array([self.declare(s) for s in listen_to]),",
            "            len(listen_to),",
            "        )",
            "",
            "    def remove_actor(self, conversation: str | int, actor: int) -> bool:",
            '        """Take an actor out of a conversation."""',
            "        return _ffi.remove_actor(self._handle, self.declare(conversation), actor) == 0",
            "",
            "    def actor_position(self, conversation: str | int, actor: int) -> int:",
            '        """Where an actor is standing."""',
            "        return _ffi.get_actor_position(self._handle, self.declare(conversation), actor)",
            "",
            "    def actor_channel(self, conversation: str | int, actor: int) -> str:",
            '        """Which channel an actor is on."""',
            "        return self.text(_ffi.get_actor_channel(self._handle, self.declare(conversation), actor))",
            "",
            "    def actor_flags(self, conversation: str | int, actor: int) -> ActorFlags:",
            '        """The flags on an actor."""',
            "        return ActorFlags(_ffi.get_actor_flags(self._handle, self.declare(conversation), actor).bits)",
            "",
            "    def actor_priority(self, conversation: str | int, actor: int) -> int:",
            '        """The priority on an actor."""',
            "        return _ffi.get_actor_priority(self._handle, self.declare(conversation), actor)",
            "",
            "    def actor_additional(self, conversation: str | int, actor: int, key: str | int) -> Any:",
            '        """Additional data on an actor, under a key."""',
            "        return self._variant(",
            "            _ffi.get_actor_additional(",
                "                self._handle, self.declare(conversation), actor, self.declare(key)",
            "            )",
            "        )",
            "",
            "    def choose(self, conversation: str | int, actor: int, index: int) -> bool:",
            '        """Take a choice on the hub an actor is standing on."""',
            "        return _ffi.select_choice(self._handle, self.declare(conversation), actor, index) == 0",
            "",
            "    def warp(",
            "        self,",
            "        conversation: str | int,",
            "        actor: int,",
            "        target: int,",
            "        *,",
            "        channel: str | int | None = None,",
            "        priority: int = 0,",
            "    ) -> bool:",
            '        """Move an actor as though it had walked there."""',
            "        return _ffi.force_warp(",
            "            self._handle,",
            "            self.declare(conversation),",
            "            actor,",
            "            target,",
            "            NAME_NONE if channel is None else self.declare(channel),",
            "            priority,",
            "        ) == 0",
            "",
            "    def teleport(",
            "        self,",
            "        conversation: str | int,",
            "        actor: int,",
            "        target: int,",
            "        *,",
            "        channel: str | int | None = None,",
            "        priority: int = 0,",
            "    ) -> bool:",
            '        """Move an actor straight there."""',
            "        return _ffi.force_position(",
            "            self._handle,",
            "            self.declare(conversation),",
            "            actor,",
            "            target,",
            "            NAME_NONE if channel is None else self.declare(channel),",
            "            priority,",
            "        ) == 0",
            "",
        ]

    def section_state(self) -> list[str]:
        return [
            "    # --- story state " + "-" * 46,
            "",
            "    def get(self, conversation: str | int, key: str | int) -> Any:",
            '        """A story state value, or a zero variant if it was never written."""',
            "        return self._variant(",
            "            _ffi.get_story_state(self._handle, self.declare(conversation), self.declare(key))",
            "        )",
            "",
            "    def set(self, conversation: str | int, key: str | int, value: Any) -> None:",
            '        """Write a story state value."""',
            "        _ffi.set_story_state(",
            "            self._handle, self.declare(conversation), self.declare(key), self._make_variant(value)",
            "        )",
            "",
            "    def state_json(self, conversation: str | int) -> str:",
            '        """The whole story state as a JSON document."""',
            "        return _take(_ffi.read_state_json, self._handle, self.declare(conversation)).decode('utf-8')",
            "",
            "    def snapshot_json(self, conversation: str | int) -> str:",
            '        """The story state and every actor, as a JSON document."""',
            "        return _take(_ffi.read_snapshot_json, self._handle, self.declare(conversation)).decode('utf-8')",
            "",
            "    # --- variants " + "-" * 52,
            "",
            "    def variant_int(self, value: int) -> Any:",
            '        """Build the variant the engine reads back as an integer."""',
            "        return _ffi.make_variant_int(int(value))",
            "",
            "    def variant_float(self, value: float) -> Any:",
            '        """Build the variant the engine reads back as a float."""',
            "        return _ffi.make_variant_float(float(value))",
            "",
            "    def variant_bool(self, value: bool) -> Any:",
            '        """Build the variant the engine reads back as an integer."""',
            "        return _ffi.make_variant_bool(bool(value))",
            "",
            "    def variant_name(self, text: str | int) -> Any:",
            '        """Build the variant the engine reads back as a name."""',
            "        return _ffi.make_variant_string(self.declare(text))",
            "",
            "    def _make_variant(self, value: Any) -> Any:",
            '        """Build the variant standing for an ordinary Python value."""',
            "        if isinstance(value, bool):",
            "            return self.variant_bool(value)",
            "        if isinstance(value, int):",
            "            return self.variant_int(value)",
            "        if isinstance(value, float):",
            "            return self.variant_float(value)",
            "        if isinstance(value, str):",
            "            return self.variant_name(value)",
            "        raise TypeError(",
            "            type(value).__name__ + ' is not a value the engine can hold'",
            "        )",
            "",
            "    def _variant(self, variant: Any) -> Any:",
            '        """Read a variant as the Python value it stands for."""',
            "        kind = variant.tag",
            "        if kind == LouderVariant_Tag.Int:",
            "            return variant.int_",
            "        if kind == LouderVariant_Tag.Float:",
            "            return variant.float_",
            "        return self.text(variant.string)",
            "",
        ]

    def section_events(self) -> list[str]:
        lines = [
            "    # --- events " + "-" * 53,
            "",
            "    def events(self) -> Iterator[Event]:",
            '        """Every queued event, oldest first, leaving the queue alone."""',
            "        index = 0",
            "        while True:",
            "            found = _ffi.get_event(self._handle, index)",
            "            if found.tag != LouderOption_LouderEvent_Tag.Some_LouderEvent:",
            "                return",
            "            yield self._event(found.some)",
            "            index += 1",
            "",
            "    def drain_events(self) -> list[Event]:",
            '        """Every queued event, oldest first, and empty the queue."""',
            "        out = list(self.events())",
            "        _ffi.flush_events(self._handle)",
            "        return out",
            "",
            "    def _event(self, raw: Any) -> Event:",
            '        """A raw event, with its names decoded and its tag read."""',
            "        kind = EventData_KINDS.get(raw.data.tag, str(raw.data.tag))",
            "        return Event(",
            "            conversation=self.text(raw.conversation),",
            "            channel=self.text(raw.channel),",
            "            actor=raw.actor,",
            "            kind=kind,",
            "            value=self._event_value(kind, raw.data),",
            "        )",
            "",
            "    def _event_value(self, kind: str, data: Any) -> Any:",
            '        """The arm the tag names, read as an ordinary Python value."""',
        ]
        arms = self.abi.unions.get("EventData")
        if arms is not None:
            for _value, name, variant in self.abi.union_arms(arms):
                if variant is None:
                    # An arm that carries nothing, so there is nothing to read.
                    # It is still reported, by name, through `Event.kind`.
                    continue
                lines.extend(self.event_arm(name, variant.type))
        lines.append("        return None")
        return lines

    def event_arm(self, variant: str, type: Type) -> list[str]:
        if variant in ("RegularMotion", "Warp", "ForceMotion"):
            return [
                "        if kind == " + repr(variant) + ":",
                "            arm = data." + snake(variant),
                "            return Motion(",
                "                source_position=arm.source_position,",
                "                target_position=arm.target_position,",
                "                choice_index=arm.choice_index,",
                "                background=bool(arm.background),",
                "            )",
            ]
        if variant == "ChoiceAppearance":
            return [
                "        if kind == 'ChoiceAppearance':",
                "            arm = data.choice_appearance",
                "            return ChoiceAppearance(",
                "                position=arm.position,",
                "                choice_index=arm.choice_index,",
                "            )",
            ]
        if variant == "AdditionalDataChange":
            return ["        if kind == 'AdditionalDataChange':", "            return self.text(data.additional_data_change.key)"]
        if variant == "Condition":
            return [
                "        if kind == 'Condition':",
                "            return (",
                "                self.text(data.condition.condition),",
                "                self._variant(data.condition.outcome),",
                "            )",
            ]
        if variant == "LoadError":
            return [
                "        if kind == 'LoadError':",
                "            return LoadFailure(",
                "                position=data.load_error.position,",
                "                error=LoadError(data.load_error.error),",
                "            )",
            ]
        target = type.name if isinstance(type, Named) else str(type)
        if target in ("LouderName", "LouderExpression", "LouderCallback"):
            return ["        if kind == " + repr(variant) + ":", "            return self.text(data." + snake(variant) + ")"]
        return ["        if kind == " + repr(variant) + ":", "            return data." + snake(variant)]

    def section_asks(self) -> list[str]:
        return [
            "    # --- asks " + "-" * 55,
            "",
            "    def asks(self) -> Iterator[Ask]:",
            '        """Every pending ask. The order is not stable."""',
            "        index = 0",
            "        while True:",
            "            found = _ffi.get_ask(self._handle, index)",
            "            if found.tag != LouderOption_LouderAsk_Tag.Some_LouderAsk:",
            "                return",
            "            raw = found.some",
            "            yield Ask(",
            "                conversation=self.text(raw.conversation),",
            "                channel=self.text(raw.channel),",
            "                actor=raw.actor,",
            "                ask=self.text(raw.ask),",
            "            )",
            "            index += 1",
            "",
            "    def answer(self, index: int, verdict: int) -> bool:",
            '        """Answer the ask at an index. Answering shifts the ones after it."""',
            "        return _ffi.answer_ask(self._handle, index, verdict) == 0",
            "",
        ]

    def section_raw(self) -> list[str]:
        return [
            "    # --- everything the generator did not wrap " + "-" * 18,
            "",
            "    def call(self, name: str, *args: Any) -> Any:",
            '        """Call any entry point by name, with the signatures of the raw layer."""',
            "        entry = getattr(_ffi, name)",
            "        return entry(self._handle, *args) if args else entry()",
            "",
            "    def push_event(self, event: Any) -> LouderError:",
            '        """Put an event on the queue, for testing."""',
            "        return LouderError(_ffi.test_push_event(self._handle, event))",
            "",
            "    def push_ask(self, ask: Any) -> LouderError:",
            '        """Put an ask in the cache, for testing."""',
            "        return LouderError(_ffi.push_ask(self._handle, ask))",
            "",
        ]

    # ------------------------------------------------------------------
    # The shared machinery, written out whole
    # ------------------------------------------------------------------

    API_HELPERS = '''
NAME_NONE = LOUDER_NAME_NONE
"""The id of nothing: what a failed lookup, or an unset name, comes back as."""

ACTOR_NONE = LOUDER_ACTOR_NONE
"""The id of no actor, which is what an actor outside the flow reads as."""

BLANK_POSITION = -1
"""The position of no hub, and where a cursor over hubs starts."""


@dataclass
class Event:
    """One thing that happened, with the names in it already decoded."""

    conversation: str
    """The conversation it happened in."""

    channel: str
    """The channel it happened on."""

    actor: int
    """The actor it happened to, or ACTOR_NONE."""

    kind: str
    """Which of the engine kinds of event this is."""

    value: Any
    """Whatever that kind carries, or None where it carries nothing."""


@dataclass
class Ask:
    """A question the engine has asked the host and is waiting on."""

    conversation: str
    channel: str
    actor: int
    ask: str
    """The name of the ask, which is what the host acts on."""


_NAME_BUFFER = 4096
_JSON_BUFFER = 1 << 16


def _bytes(value: Any) -> bytes:
    if isinstance(value, str):
        return value.encode("utf-8")
    if isinstance(value, (bytes, bytearray, memoryview)):
        return bytes(value)
    raise TypeError(type(value).__name__ + " is neither text nor bytes")


def _with(engine: "Louder", value: Any) -> tuple[Any, int]:
    """A buffer and a length, which is how every string crosses the boundary.

    The buffer is an array of the byte type the prototypes declare rather than
    a string buffer: `ctypes` will not quietly pass one where the other is
    wanted, which is the right way round.
    """
    raw = _bytes(value)
    buffer = (ctypes.c_uint8 * max(len(raw), 1))()
    ctypes.memmove(buffer, raw, len(raw))
    return buffer, len(raw)


def _take(call: Any, *args: Any, size: int = _NAME_BUFFER) -> bytes:
    """Call something that fills a buffer, and return what it wrote.

    The engine truncates rather than refusing, and says how much fitted, so the
    only way to be sure of the whole answer is to ask again with more room. Two
    calls is the normal case; the loop is what makes a long document safe.
    """
    while True:
        buffer = (ctypes.c_uint8 * size)()
        written = call(*args, buffer, size - 1)
        if written < 0:
            raise LouderFailure(call.__name__ + " reported " + str(written))
        if written < size - 1:
            return bytes(memoryview(buffer).cast("B")[:written])
        size *= 4


def _name_array(values: Sequence[int]) -> Any:
    """A list of name ids as the array the engine reads."""
    array = (ctypes.c_int32 * max(len(values), 1))()
    for index, value in enumerate(values):
        array[index] = int(value)
    return array


def _bits(type_name: str, value: int) -> Any:
    """A flags value as the struct the engine takes.

    The convenience layer has the flags as an `IntFlag`, which is what a caller
    wants to write; the boundary has them as a struct wrapping the bits. This
    is the one crossing between the two, and it is the only place it happens.
    """
    built = getattr(_ffi, type_name)()
    built.bits = int(value)
    return built


class Louder:
    """One engine context.

    Owns the name table, every conversation loaded into it, and the event queue.
    Closing it releases all of that at once.
    """

    __slots__ = ("_handle", "_closed")

    def __init__(self, library: Any = None, path: str | None = None) -> None:
        self._handle = (library or _ffi.load_library(path)).init_louder()
        if not self._handle:
            raise LouderFailure("the engine could not allocate a context")
        self._closed = False

    def close(self) -> None:
        if not self._closed:
            self._closed = True
            _ffi.deinit_louder(self._handle)

    def __enter__(self) -> "Louder":
        return self

    def __exit__(self, *_: Any) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:
            pass

    @property
    def handle(self) -> Any:
        """The raw context pointer, for anything this layer does not wrap."""
        return self._handle
'''

    # ------------------------------------------------------------------
    # Packaging and prose
    # ------------------------------------------------------------------

    def render_readme(self) -> list[str]:
        return (
            [
                "# Louder, for Python",
                "",
                "Generated from the C header of the `louder_rust` engine. Do not edit;",
                "run `python engine/bindings/generate.py` and this directory comes back.",
                "",
                "Two modules:",
                "",
                "- `louder._ffi` is the boundary as the header writes it: `ctypes`",
                "  structures, the constants, and the entry points with their",
                "  signatures attached.",
                "- `louder` is the same engine in Python terms: names are `str`,",
                "  buffers are handled for you, a variant is `int | float | str`, and",
                "  the `next_*` family is a generator.",
                "",
                "```python",
                "from louder import Louder",
                "",
                "with Louder() as engine:",
                "    engine.load_config(open('speakers.json').read())",
                "    engine.load_conversation('chat', open('chat.json').read())",
                "",
                "    actor = engine.add_actor('chat', 0, speak_as=['A'])",
                "    engine.choose('chat', actor, 0)",
                "",
                "    for event in engine.drain_events():",
                "        print(event.kind, event.value)",
                "",
                "    where = engine.actor_position('chat', actor)",
                "    print(engine.hub_text('chat', where))",
                "    print(engine.get('chat', 'mood'))",
                "```",
                "",
                "## Loading the engine",
                "",
                "The module looks for `louder_rust.dll`, `liblouder_rust.so` or",
                "`liblouder_rust.dylib` beside the package and then on the platform",
                "search path. Build it with:",
                "",
                "```",
                "cargo build -p louder_rust --release",
                "```",
                "",
                "Point `LOUDER_LIBRARY` at the file, or pass a path to `Louder(path=...)`,",
                "if it lives somewhere else.",
                "",
                "## What was generated",
                "",
            ]
            + self.count_lines()
            + [""]
        )

    def counts(self) -> str:
        shapes = self.shapes()
        return (
            str(shapes["functions"])
            + " functions, "
            + str(shapes["structs"])
            + " structs, "
            + str(shapes["unions"])
            + " tagged unions, "
            + str(shapes["enums"])
            + " enums, "
            + str(shapes["opaques"])
            + " opaque handle and "
            + str(shapes["constants"])
            + " constants"
        )

    def count_lines(self) -> list[str]:
        return ["- " + self.counts()]

    def render_pyproject(self) -> list[str]:
        return [
            "[build-system]",
            'requires = ["setuptools>=68"]',
            'build-backend = "setuptools.build_meta"',
            "",
            "[project]",
            'name = "louder"',
            'version = "0.1.0"',
            'description = "Bindings for the Louder dialogue engine"',
            'requires-python = ">=3.10"',
            "",
            "[tool.setuptools.packages.find]",
            'include = ["louder*"]',
            "",
            "[tool.setuptools.package-data]",
            'louder = ["py.typed"]',
        ]


def snake(name: str) -> str:
    """`RegularMotion` becomes `regular_motion`, which is how cbindgen names it."""
    out: list[str] = []
    for index, char in enumerate(name):
        if char.isupper() and index:
            out.append("_")
        out.append(char.lower())
    return "".join(out)


def title_words(name: str) -> str:
    """`MALFORMED_JSON` becomes `MalformedJson`, which is how it reads in Rust."""
    return "".join(word[:1].upper() + word[1:].lower() for word in name.split("_") if word)
