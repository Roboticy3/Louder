"""Go, reaching the engine through cgo.

cgo wants a C header to include and a library to link, and both are written out
alongside the Go: `louder.h` is the same header the C backend produces, and the
import library is copied in so the package builds as it ships. On top of that
`package louder` gives an owning type, real strings, and a `range` loop for each
of the cursors, which is the part Go does better than C.
"""

from __future__ import annotations

from pathlib import Path

from ..ir import Func, Named, Pointer, Scalar
from ..naming import Names, camel, pascal, snake
from .base import Emitted
from .c import CBackend


class GoBackend(CBackend):
    name = "go"
    label = "Go"
    tier = "solid"
    convenience = "full"
    needs_library = True
    extension = "go"
    include_guard = "LOUDER_H"
    header_file = "louder.h"
    own_banner = [
        "/* Bindings for Go, generated from the C header of the Louder engine.",
        " *",
        " * One difference from the header the engine ships: this one compiles. C keeps",
        " * struct tags, typedef names, and enum constants in a single namespace, and the",
        " * engine has a variant of `EventData_Tag` named `ChoiceAppearance` alongside a",
        " * struct of the same name. A constant that would redeclare something else in the",
        " * file is spelled with the enum's name in front of it; nothing else is renamed.",
        " *",
        " * This header is what cgo includes. The Go itself is in `louder.go`, and it is",
        " * the file to read first.",
        " *",
        " * Do not edit. Run `python engine/bindings/generate.py` and this file comes back.",
        " */",
    ]

    def __init__(self, abi, layout, options):
        super().__init__(abi, layout, options)
        self.go = Names("go", type_case=pascal, func_case=camel, field_case=snake,
                        const_case=snake, arg_case=snake, suffix="_")

    def build(self, out: Emitted) -> None:
        out.add_lines("louder/louder.h", self.render())
        out.add_lines("louder/louder.go", self.render_go())
        out.add_lines("go.mod", self.render_gomod())
        out.add("cmd/walk/main.go", self.runtime("go/walk.go").read_text(encoding="utf-8"))
        out.add_lines("README.md", self.render_readme())
        # The import library is what the linker actually opens on Windows, and
        # it is a different file from the DLL, so copy both. MinGW's ld finds a
        # library by a pattern rather than by path, so it is named the pattern.
        library = self.options.library
        if library is not None:
            import_lib = library.with_name(library.name + ".lib")
            if import_lib.is_file():
                out.add_binary("louder/liblouder_rust.dll.a", import_lib.read_bytes())

    def epilogue(self) -> list[str]:
        # The convenience layer is Go, so the C header stays the boundary.
        return []

    # -- go.mod ------------------------------------------------------------

    def render_gomod(self) -> list[str]:
        return [
            "// Generated from the C header of the Louder engine.",
            "module louder",
            "",
            "go 1.25",
            "",
        ]

    # -- the go file ---------------------------------------------------------

    def render_go(self) -> list[str]:
        lines = [
            "// Bindings for Go, generated from the C header of the Louder engine.",
            "//",
            "// Do not edit. Run `python engine/bindings/generate.py` and this file comes back.",
            "//",
            "// One engine context and everything that comes with it.",
            "",
            "package louder",
            "",
            "// #cgo windows LDFLAGS: ${SRCDIR}/louder_rust.dll.lib",
            "// #cgo linux LDFLAGS: -L${SRCDIR} -llouder_rust",
            "// #cgo darwin LDFLAGS: -L${SRCDIR} -llouder_rust",
            '// #include "louder.h"',
            "import \"C\"",
            "",
            "import (",
            "    \"errors\"",
            "    \"fmt\"",
            "    \"iter\"",
            "    \"unsafe\"",
            ")",
            "",
        ]
        lines.extend(self.go_types())
        lines.extend(self.go_constants())
        lines.extend(self.go_raw())
        lines.extend(self.go_engine())
        lines.extend(self.go_cursors())
        return lines

    # -- the types ---------------------------------------------------------

    def go_types(self) -> list[str]:
        lines = [
            "// The boundary's types, named as the header names them.",
            "//",
            "// Every one is an alias of the C type rather than a new type, because",
            "// the two are the same thing and a caller should not have to convert.",
            "",
            "type Context = C.LouderContext",
        ]
        for name in sorted(self.abi.aliases):
            lines.append("type " + self.go.type(name) + " = C." + name)
        for name in sorted(self.abi.enums):
            lines.append("type " + self.go.type(name) + " = C." + name)
        for name in sorted(self.abi.structs):
            if name in self.abi.union_bodies:
                continue
            lines.append("type " + self.go.type(name) + " = C." + name)
        for name in sorted(self.abi.unions):
            lines.append("type " + self.go.type(name) + " = C." + name)
        lines.append("")
        return lines

    def go_constants(self) -> list[str]:
        lines = [
            "// The constants and the values of the enums.",
            "const (",
        ]
        for const in self.abi.consts:
            if const.name.startswith("LOUDER_"):
                # Kept as the header spells it: these are the engine's own names
                # for its own values, and `louder.LOUDER_NAME_NONE` reads as such.
                lines.append("    " + const.name + " = C." + const.name)
        lines.append(")")
        lines.append("")
        for name in sorted(self.abi.enums):
            enum = self.abi.enums[name]
            prefix = self.go_enum_prefix(name)
            lines.append("const (")
            for variant, _explicit in enum.variants:
                member = self.enum_member(name, variant)
                lines.append("    " + prefix + pascal(variant) + " = C." + member)
            lines.append(")")
            lines.append("")
        # The bitflags, with one constant per flag so they read as flags rather
        # than as the one-field struct cbindgen lowered them to.
        for name in sorted(self.abi.bitflags):
            flag = self.abi.bitflags[name]
            lines.append("// " + (flag.doc or "The flags of a " + name + "."))
            lines.append("const (")
            for variant, _value in flag.variants:
                lines.append("    " + self.go.type(name) + pascal(variant) + " = C." + name + "_" + variant)
            lines.append(")")
            lines.append("")
        return lines

    def go_enum_prefix(self, enum_name: str) -> str:
        """What a Go constant for one of this enum's variants starts with.

        `LouderVariant_Tag` variants are `VariantInt`, `VariantFloat` and so on,
        which is how the values read at a call site without colliding with the
        C names they stand for.
        """
        stem = enum_name
        for ending in ("_Tag", "Tag"):
            if stem.endswith(ending):
                stem = stem[: -len(ending)]
                break
        stem = stem.replace("LouderOption_", "")
        stem = stem.replace("Louder", "")
        if not stem:
            stem = enum_name
        return stem

    # -- the raw layer -----------------------------------------------------

    def go_raw(self) -> list[str]:
        """One Go function per entry point, capitalised and argument-for-argument.

        These are what a caller reaches for when the ergonomic layer does not
        wrap something. They convert rather than interpret: a `size_t` is a
        `C.size_t`, a string is a pointer and a length, and nothing is checked
        that the engine does not check itself.
        """
        lines = [
            "// The raw entry points, one per function in the header.",
            "",
        ]
        for func in self.abi.funcs:
            lines.extend(self.go_raw_one(func))
        return lines

    def go_raw_one(self, func: Func) -> list[str]:
        name = pascal(func.name)
        params = []
        for index, param in enumerate(func.params):
            params.append(self.go.arg(param.name, index) + " " + self.go_raw_type(param.type))
        call_args = ", ".join(self.go_raw_arg(param.name, index, func.params[index].type) for index, param in enumerate(func.params))
        ret = self.go_raw_return(func.ret)
        lines = []
        if func.doc:
            for line in " ".join(func.doc.split()).split(". ")[:1]:
                lines.append("// " + line + ".")
        else:
            lines.append("// " + name + " calls " + func.name + ".")
        signature = "func " + name + "(" + ", ".join(params) + ")" + (" " + ret if ret else "") + " {"
        if ret:
            lines.append(signature)
            lines.append("    return " + ret + "(C." + func.name + "(" + call_args + "))")
        else:
            lines.append(signature)
            lines.append("    C." + func.name + "(" + call_args + ")")
        lines.append("}")
        lines.append("")
        return lines

    def go_raw_type(self, type) -> str:
        resolved = self.abi.resolve(type)
        if isinstance(resolved, Pointer):
            inner = self.abi.resolve(resolved.to)
            if isinstance(inner, Named) and inner.name == "LouderContext":
                return "*Context"
            if isinstance(inner, Named):
                return "*" + self.go.type(inner.name)
            if isinstance(inner, Scalar):
                return "*C." + inner.name
            return "unsafe.Pointer"
        if isinstance(resolved, Named):
            return self.go.type(resolved.name)
        if isinstance(resolved, Scalar):
            return self.go_scalar(resolved)
        return "int32"

    def go_scalar(self, scalar: Scalar) -> str:
        return {
            "void": "",
            "bool": "bool",
            "int8_t": "int8",
            "uint8_t": "byte",
            "int16_t": "int16",
            "uint16_t": "uint16",
            "short": "int16",
            "int32_t": "int32",
            "uint32_t": "uint32",
            "int": "int32",
            "unsigned": "uint32",
            "float": "float32",
            "int64_t": "int64",
            "uint64_t": "uint64",
            "long long": "int64",
            "double": "float64",
            "size_t": "int",
            "ptrdiff_t": "int",
        }.get(scalar.name, "int32")

    def c_base(self, type) -> str:
        """The C spelling cgo knows for a type behind a pointer."""
        resolved = self.abi.resolve(type)
        if isinstance(resolved, Named):
            if resolved.name == "void":
                return "void"
            return resolved.name
        if isinstance(resolved, Scalar):
            return resolved.name
        return "uint8_t"

    def go_raw_arg(self, name: str, index: int, type) -> str:
        resolved = self.abi.resolve(type)
        arg = self.go.arg(name, index)
        if isinstance(resolved, Pointer):
            return arg
        if isinstance(resolved, Scalar):
            c_name = resolved.name
            if c_name == "bool":
                return "C.bool(" + arg + ")"
            if c_name == "size_t":
                return "C.size_t(" + arg + ")"
            if c_name in ("int8_t", "uint8_t", "int16_t", "uint16_t", "int32_t", "uint32_t", "int64_t", "uint64_t"):
                return "C." + c_name + "(" + arg + ")"
            if c_name == "float":
                return "C.float(" + arg + ")"
            if c_name == "double":
                return "C.double(" + arg + ")"
            return arg
        # Aggregates pass through as themselves: cgo moves a struct by value.
        return arg

    def go_raw_return(self, type) -> str:
        resolved = self.abi.resolve(type)
        if isinstance(resolved, Scalar) and resolved.name == "void":
            return ""
        if isinstance(resolved, Named):
            return self.go.type(resolved.name)
        if isinstance(resolved, Scalar):
            return self.go_scalar(resolved)
        if isinstance(resolved, Pointer):
            return "*C.uint8_t"
        return "int32"

    def go_engine(self) -> list[str]:
        return [
            "// Louder is one engine context.",
            "//",
            "// It owns the name table, every conversation, and the event queue.",
            "// Close it when you are done; there is no finalizer, because the engine",
            "// cannot be closed from the garbage collector's thread.",
            "type Louder struct {",
            "    ctx *C.LouderContext",
            "}",
            "",
            "// Open starts an engine context.",
            "func Open() (*Louder, error) {",
            "    ctx := C.init_louder()",
            "    if ctx == nil {",
            '        return nil, errors.New("the engine did not start")',
            "    }",
            "    return &Louder{ctx: ctx}, nil",
            "}",
            "",
            "// Close releases the context and everything it holds.",
            "func (l *Louder) Close() {",
            "    if l.ctx != nil {",
            "        C.deinit_louder(l.ctx)",
            "        l.ctx = nil",
            "    }",
            "}",
            "",
            "// Handle is the raw context pointer, for anything this layer does not wrap.",
            "func (l *Louder) Handle() *C.LouderContext { return l.ctx }",
            "",
            "// Lookup answers the id of some text, or LOUDER_NAME_NONE if it was never",
            "// declared.",
            "func (l *Louder) Lookup(text string) LouderName {",
            "    raw, length := bytes(text)",
            "    return C.get_named_id(l.ctx, raw, length)",
            "}",
            "",
            "// Name answers the id of some text, declaring it the first time.",
            "//",
            "// declare_name refuses to make a second id for text that already has one,",
            "// returning LOUDER_NAME_NONE, so this looks first and declares second.",
            "func (l *Louder) Name(text string) LouderName {",
            "    found := l.Lookup(text)",
            "    if found != LOUDER_NAME_NONE {",
            "        return found",
            "    }",
            "    raw, length := bytes(text)",
            "    return C.declare_name(l.ctx, raw, length)",
            "}",
            "",
            "// Text answers the bytes behind an id, decoded as a Go string.",
            "//",
            "// The engine reports how many bytes fitted rather than refusing, so the",
            "// only way to be sure of the whole answer is to ask again with more room.",
            "// Two calls is the ordinary case.",
            "func (l *Louder) Text(id LouderName) string {",
            "    if id == LOUDER_NAME_NONE {",
            "        return \"\"",
            "    }",
            "    for size := 4096; ; size *= 4 {",
            "        buffer := make([]byte, size)",
            "        got := C.read_name(l.ctx, id, (*C.uint8_t)(unsafe.Pointer(&buffer[0])), C.size_t(size-1))",
            "        if got == 0 {",
            "            return \"\"",
            "        }",
            "        if int(got)+1 < size {",
            "            return string(buffer[:int(got)])",
            "        }",
            "    }",
            "}",
            "",
            "// Failure is why a document was refused.",
            "type Failure struct { Code int32 }",
            "",
            "func (f Failure) Error() string {",
            "    switch f.Code {",
            "    case LOUDER_LOAD_MALFORMED_JSON:",
            '        return "the document is not JSON"',
            "    case LOUDER_LOAD_NOT_A_LIST:",
            '        return "the document is not a list"',
            "    case LOUDER_LOAD_NOT_AN_OBJECT:",
            '        return "the document is not an object"',
            "    case LOUDER_LOAD_ALREADY_LOADED:",
            '        return "a conversation by that name is already loaded"',
            "    default:",
            '        return fmt.Sprintf("the engine refused the document (%d)", f.Code)',
            "    }",
            "}",
            "",
            "// LoadConversation loads a conversation from a JSON document.",
            "func (l *Louder) LoadConversation(name LouderName, document string) error {",
            "    raw, length := bytes(document)",
            "    if code := C.load_conversation_json(l.ctx, name, raw, length); code != LOUDER_LOAD_OK {",
            "        return Failure{Code: int32(code)}",
            "    }",
            "    return nil",
            "}",
            "",
            "// LoadConfig loads the speaker config, which is a singleton.",
            "func (l *Louder) LoadConfig(document string) error {",
            "    raw, length := bytes(document)",
            "    if code := C.load_conversation_config(l.ctx, raw, length); code != LOUDER_LOAD_OK {",
            "        return Failure{Code: int32(code)}",
            "    }",
            "    return nil",
            "}",
            "",
            "// StateJSON is the whole story state as a JSON document.",
            "func (l *Louder) StateJSON(conversation LouderName) (string, error) {",
            "    for size := 4096; ; size *= 4 {",
            "        buffer := make([]byte, size)",
            "        written := C.read_state_json(l.ctx, conversation, (*C.uint8_t)(unsafe.Pointer(&buffer[0])), C.size_t(size-1))",
            "        if written < 0 {",
            '            return "", errors.New("the conversation is not loaded")',
            "        }",
            "        if int(written)+1 < size {",
            "            return string(buffer[:int(written)]), nil",
            "        }",
            "    }",
            "}",
            "",
            "// SnapshotJSON is the story state and every actor, as a JSON document.",
            "func (l *Louder) SnapshotJSON(conversation LouderName) (string, error) {",
            "    for size := 4096; ; size *= 4 {",
            "        buffer := make([]byte, size)",
            "        written := C.read_snapshot_json(l.ctx, conversation, (*C.uint8_t)(unsafe.Pointer(&buffer[0])), C.size_t(size-1))",
            "        if written < 0 {",
            '            return "", errors.New("the conversation is not loaded")',
            "        }",
            "        if int(written)+1 < size {",
            "            return string(buffer[:int(written)]), nil",
            "        }",
            "    }",
            "}",
            "",
            "// Variant is one value from the story state, decoded.",
            "type Variant struct {",
            "    Kind LouderVariantTag",
            "    Int   int32",
            "    Float float32",
            "    Name  LouderName",
            "}",
            "",
            "// Variant answers the story state value under key, decoded.",
            "func (l *Louder) Variant(conversation LouderName, key string) Variant {",
            "    raw := C.get_story_state(l.ctx, conversation, l.Name(key))",
            "    out := Variant{Kind: LouderVariantTag(raw.tag)}",
            "    switch out.Kind {",
            "    case VariantInt:",
            "        out.Int = *(*int32)(unsafe.Pointer(&raw.body[0]))",
            "    case VariantFloat:",
            "        out.Float = *(*float32)(unsafe.Pointer(&raw.body[0]))",
            "    case VariantString:",
            "        out.Name = *(*LouderName)(unsafe.Pointer(&raw.body[0]))",
            "    }",
            "    return out",
            "}",
            "",
            "// SetInt writes an integer story state value.",
            "func (l *Louder) SetInt(conversation LouderName, key string, value int32) {",
            "    C.set_story_state(l.ctx, conversation, l.Name(key), C.make_variant_int(C.int32_t(value)))",
            "}",
            "",
            "// SetFloat writes a float story state value.",
            "func (l *Louder) SetFloat(conversation LouderName, key string, value float32) {",
            "    C.set_story_state(l.ctx, conversation, l.Name(key), C.make_variant_float(C.float(value)))",
            "}",
            "",
            "// SetText writes a text story state value, stored as the interned name.",
            "func (l *Louder) SetText(conversation LouderName, key string, value string) {",
            "    C.set_story_state(l.ctx, conversation, l.Name(key), C.make_variant_string(l.Name(value)))",
            "}",
            "",
            "// bytes hands a Go string over as the pointer and length the engine wants.",
            "//",
            "// The empty string has no first element to point at, and the engine accepts",
            "// a null pointer when the length is zero.",
            "func bytes(text string) (*C.uint8_t, C.size_t) {",
            "    if len(text) == 0 {",
            "        return nil, 0",
            "    }",
            "    return (*C.uint8_t)(unsafe.Pointer(unsafe.StringData(text))), C.size_t(len(text))",
            "}",
            "",
        ]

    def go_cursors(self) -> list[str]:
        """One `range`-able walk per cursor.

        A cursor is any entry point whose last parameter is named `start`: the
        engine hands one element at a time and the caller passes the previous one
        back, until the end value comes back. Written as an `iter.Seq` each walk
        reads as a for-range loop, which is what Go has instead of iterators.
        """
        lines: list[str] = []
        for func in self.cursors():
            lines.extend(self.go_cursor(func))
        return lines

    def go_cursor(self, func: Func) -> list[str]:
        stem = func.name
        for prefix in ("get_next_", "next_"):
            if stem.startswith(prefix):
                stem = stem[len(prefix):]
                break
        name = "".join(w.capitalize() for w in stem.split("_"))
        leading = func.params[:-1]
        element = self.go.type(self.abi.resolve(func.ret).name) if isinstance(self.abi.resolve(func.ret), Named) else "int32"
        sentinel = self.none_constant(func.ret)

        params = ", ".join(self.go.arg(p.name, i) + " " + self.go_raw_type(p.type) for i, p in enumerate(leading))
        call_args = ", ".join(self.go_raw_arg(p.name, i, p.type) for i, p in enumerate(leading))

        doc = " ".join((func.doc.splitlines() or [""])[0].split())
        lines = [
            "// " + name + " walks " + stem + ".",
            "//",
            "// " + doc,
            "func (l *Louder) " + name + "(" + params + ") iter.Seq[" + element + "] {",
            "    return func(yield func(" + element + ") bool) {",
            "        for start := " + element + "(" + sentinel + "); ; {",
            "            found := C." + func.name + "(l.ctx" + (", " if call_args else "") + call_args + ", start)",
            "            if found == " + element + "(" + sentinel + ") {",
            "                return",
            "            }",
            "            if !yield(found) {",
            "                return",
            "            }",
            "            start = found",
            "        }",
            "    }",
            "}",
            "",
        ]
        return lines

    def render_readme(self) -> list[str]:
        return [
            "# Louder, from Go",
            "",
            "Generated. Do not edit; run `python engine/bindings/generate.py` instead.",
            "",
            "## Files",
            "",
            "| File | What it is |",
            "| --- | --- |",
            "| `louder/louder.go` | The package: an owning `Louder`, real strings, and a `range` loop per cursor. |",
            "| `louder/louder.h` | The C header cgo includes. |",
            "| `cmd/walk/main.go` | A complete example. |",
            "",
            "## Building",
            "",
            "The package uses cgo, so it needs a C compiler. On Windows that is",
            "MinGW gcc; the import library is copied into the package, so it links",
            "as it ships:",
            "",
            "```",
            "go build ./...",
            "```",
            "",
            "To run, `louder_rust.dll` has to be on `PATH` (or `louder_rust.so` /",
            "`louder_rust.dylib` on `LD_LIBRARY_PATH`).",
            "",
            "## The bits worth knowing",
            "",
            "- **`louder.Open()` returns an error.** The engine can refuse to start.",
            "- **`Close` is explicit.** There is no finalizer: the engine cannot be",
            "  closed from the garbage collector's thread.",
            "- **Names are `int32` handles.** `l.Name(text)` does lookup-then-declare,",
            "  because the engine refuses to make a second id for a string that has",
            "  one. Handles come from a hash, so they are not sequential.",
            "- **Every cursor is a loop.**",
            "",
            "```go",
            "for hub := range l.HubsByChannel(conversation, channel) {",
            "    // ...",
            "}",
            "```",
            "",
        ]