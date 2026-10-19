"""JavaScript bindings, in two layers.

`raw.js` is the boundary as the module actually exports it. There is no glue
code between JavaScript and the engine: the `.wasm` file is instantiated
directly and its exports called with the arguments the module's own type section
says they take. That section is read when the bindings are generated, so the
two shapes the C ABI takes on WebAssembly — an aggregate return written through
a pointer the caller supplies, and an aggregate argument copied into memory
before the call — are written down from the module rather than worked out from
rules.

`index.js` is the same engine in JavaScript terms: names are strings, buffers
size themselves, a variant is a number or a string, an event carries a kind and
a decoded body, and the `next_*` cursors are generators.
"""

from __future__ import annotations

from pathlib import Path

from ..ir import Func, Named, Pointer, Scalar, Struct, TaggedUnion, Type
from ..naming import Names, camel, snake, screaming
from .base import Backend, Emitted

_RUNTIME = Path(__file__).resolve().parent.parent.parent / "runtime" / "javascript"

_JS_SCALARS = {
    "bool": "boolean",
    "int8_t": "number",
    "uint8_t": "number",
    "int16_t": "number",
    "uint16_t": "number",
    "int32_t": "number",
    "uint32_t": "number",
    "int": "number",
    "unsigned": "number",
    "int64_t": "bigint",
    "uint64_t": "bigint",
    "float": "number",
    "double": "number",
    "size_t": "number",
    "ptrdiff_t": "number",
}

# What ends a walk over a cursor, and what the header calls it.
SENTINELS = {"LouderName": "NAME_NONE", "LouderActor": "ACTOR_NONE", "LouderPosition": "BLANK_POSITION"}

# Arguments the engine names after things that are names.
NAME_ARGUMENTS = frozenset({"conversation", "channel", "speaker", "key", "state", "text", "ask"})


class JavaScriptBackend(Backend):
    name = "javascript"
    label = "JavaScript"
    tier = "deep"
    convenience = "full"
    extension = "js"
    needs_wasm = True

    def make_names(self) -> Names:
        return Names("javascript", type_case=str, func_case=camel, field_case=camel, const_case=camel)

    def comment_prefix(self) -> str:
        return "//"

    def build(self, out: Emitted) -> None:
        out.add_lines("louder/memory.js", self.runtime("javascript/memory.js").read_text(encoding="utf-8").splitlines())
        out.add_lines("louder/raw.js", self.render_raw())
        out.add_lines("louder/index.js", self.render_index())
        out.add_lines("louder/index.d.ts", self.render_types())
        out.add_lines("louder/raw.d.ts", self.render_raw_types())
        out.add_lines("package.json", self.render_package())
        out.add_lines("README.md", self.render_readme())

    # ------------------------------------------------------------------
    # Layout tables
    # ------------------------------------------------------------------

    def sizes(self) -> dict[str, int]:
        out: dict[str, int] = {}
        for struct in self.abi.structs.values():
            out[struct.name] = self.layout.size(struct)
        for union in self.abi.unions.values():
            out[union.name] = self.layout.size(union)
        for name in sorted(self.abi.enums):
            out[name] = self.layout.size(Named(name))
        for alias in self.abi.aliases.values():
            out[alias.name] = self.layout.size(alias)
        return out

    def offsets(self) -> dict[str, dict[str, int]]:
        out: dict[str, dict[str, int]] = {}
        for struct in self.abi.structs.values():
            out[struct.name] = {f.name: f.offset for f in self.layout.fields(struct)}
        for union in self.abi.unions.values():
            body = self.abi.structs[union.name + "_Body"]
            at = self.layout.variant_offset(union, union.variants[0]) if union.variants else 0
            out[union.name] = {"tag": 0}
            out[union.name + "_Body"] = {f.name: at for f in body.fields}
        return out

    def sizes_table(self) -> list[str]:
        sizes = self.sizes()
        lines = [
            "/** How much of linear memory each declared type occupies. */",
            "export const SIZE = Object.freeze({",
        ]
        for name in sorted(sizes):
            lines.append("  " + name + ": " + str(sizes[name]) + ",")
        lines.append("});")
        lines.append("")
        return lines

    def offsets_table(self) -> list[str]:
        table = self.offsets()
        lines = [
            "/**",
            " * Where each field sits inside its type.",
            " *",
            " * Every arm of a tagged union shares one block of memory, so they are all",
            " * read from the same offset and the tag says which one means anything.",
            " */",
            "export const OFFSET = Object.freeze({",
        ]
        for name in sorted(table):
            lines.append("  " + name + ": Object.freeze({")
            for field, offset in table[name].items():
                lines.append("    " + field + ": " + str(offset) + ",")
            lines.append("  }),")
        lines.append("});")
        lines.append("")
        return lines

    # ------------------------------------------------------------------
    # Codecs
    # ------------------------------------------------------------------

    def read_expr(self, type: Type, offset: str) -> str:
        if isinstance(type, Pointer):
            return "view.getInt32(" + offset + ", true)"
        if isinstance(type, Scalar):
            if type.name == "bool":
                return "(view.getUint8(" + offset + ") !== 0)"
            if type.name in ("float", "double"):
                return "view.getFloat32(" + offset + ", true)"
            if type.name in ("int64_t", "uint64_t"):
                return "view.getBigInt64(" + offset + ", true)"
            return "view.getInt32(" + offset + ", true)"
        resolved = self.abi.resolve(type)
        if isinstance(resolved, Scalar):
            return self.read_expr(resolved, offset)
        return "read" + self.underlying(type) + "(view, " + offset + ")"

    def write_expr(self, type: Type, offset: str, value: str, indent: str = "  ") -> str:
        if isinstance(type, Pointer):
            return indent + "view.setInt32(" + offset + ", " + value + ", true);"
        if isinstance(type, Scalar):
            if type.name == "bool":
                return indent + "view.setUint8(" + offset + ", " + value + " ? 1 : 0);"
            if type.name in ("float", "double"):
                return indent + "view.setFloat32(" + offset + ", " + value + ", true);"
            if type.name in ("int64_t", "uint64_t"):
                return indent + "view.setBigInt64(" + offset + ", BigInt(" + value + "), true);"
            return indent + "view.setInt32(" + offset + ", " + value + ", true);"
        resolved = self.abi.resolve(type)
        if isinstance(resolved, Scalar):
            return self.write_expr(resolved, offset, value, indent)
        return indent + "write" + self.underlying(type) + "(view, " + offset + ", " + value + ");"

    def underlying(self, type: Type) -> str:
        resolved = self.abi.resolve(type)
        return resolved.name if isinstance(resolved, Named) else str(resolved)

    def codec_for_struct(self, struct: Struct) -> list[str]:
        name = struct.name
        fields = self.layout.fields(struct)
        lines = [
            "",
            self.doc_block(struct.doc, "The " + name + " type."),
            "export function read" + name + "(view, at) {",
            "  return {",
        ]
        for placed in fields:
            # Offsets are relative to `at`, not absolute: the same struct is
            # read at the top of a buffer and from inside three others.
            lines.append("    " + camel(placed.name) + ": " + self.read_expr(placed.type, "at + " + str(placed.offset)) + ",")
        lines.append("  };")
        lines.append("}")
        lines.append("")
        lines.append("export function write" + name + "(view, at, value) {")
        for placed in fields:
            lines.append(self.write_expr(placed.type, "at + " + str(placed.offset), "value." + camel(placed.name)))
        lines.append("}")
        return lines

    def codec_for_union(self, union: TaggedUnion) -> list[str]:
        """A tagged union reads as its tag plus every arm, decoded.

        Every arm is read from the same offset because they share the same
        memory; the tag says which one means anything. Decoding them all is a
        few extra reads on structures this small, and it removes the mistake
        where the wrong arm is read for a tag a newer engine produced.
        """
        name = union.name
        arms = self.abi.union_arms(union)
        body = self.layout.variant_offset(union, union.variants[0]) if union.variants else 0

        lines = [
            "",
            self.doc_block(union.doc, "The " + name + " type."),
            "export function read" + name + "(view, at) {",
            "  return {",
            "    tag: view.getInt32(at + OFFSET." + name + ".tag, true),",
        ]
        for _value, variant, carrier in arms:
            if carrier is None:
                continue
            lines.append("    " + camel(snake(variant)) + ": " + self.read_expr(carrier.type, "at + " + str(body)) + ",")
        lines.append("  };")
        lines.append("}")
        lines.append("")
        lines.append("export function write" + name + "(view, at, value) {")
        lines.append("  view.setInt32(at + OFFSET." + name + ".tag, value.tag, true);")
        for _value, variant, carrier in arms:
            if carrier is None:
                continue
            field = "value." + camel(snake(variant))
            lines.append("  if (" + field + " !== undefined) {")
            lines.append("  " + self.write_expr(carrier.type, "at + " + str(body), field, indent="  ").rstrip(";"))
            lines.append("  }")
        lines.append("}")
        return lines

    def one_line(self, text: str) -> str:
        return " ".join((text or "").split())

    # ------------------------------------------------------------------
    # The raw layer
    # ------------------------------------------------------------------

    def render_raw(self) -> list[str]:
        lines: list[str] = []
        lines.extend(self.banner())
        lines.extend(
            [
                "/**",
                " * The C boundary, as the WebAssembly module actually exports it.",
                " *",
                " * There is no glue code between here and the engine. The module is",
                " * instantiated directly and its exports called with the arguments its own",
                " * type section says they take, which is where the layouts below and the",
                " * signatures in this file come from.",
                " */",
                "",
                'import {',
                '  fromBytes,',
                '  put,',
                '  readInto,',
                '  reserve,',
                '  take,',
                '  toBytes,',
                '  view,',
                '  withBytes,',
                '} from "./memory.js";',
                "",
                "",
            ]
        )
        lines.extend(self.sizes_table())
        lines.extend(self.offsets_table())
        lines.extend(self.name_tables())

        for struct in self.ordered_structs():
            lines.extend(self.codec_for_struct(struct))
        for union in self.ordered_unions():
            lines.extend(self.codec_for_union(union))

        lines.extend(self.constants_js())
        lines.extend(self.raw_class())
        lines.extend(
            [
                "/** Instantiate an engine module and hand back its raw layer. */",
                "export async function instantiate(source) {",
                "  const { instance } = await WebAssembly.instantiate(source, {});",
                "  return new Raw(instance);",
                "}",
                "",
                "/** Load a `.wasm` file, an `ArrayBuffer`, or anything `fetch` returns. */",
                "export async function load(source) {",
                "  if (typeof source === 'string' || source instanceof URL) {",
                "    return instantiate(await (await fetch(source)).arrayBuffer());",
                "  }",
                "  return instantiate(source);",
                "}",
            ]
        )
        return lines

    def name_tables(self) -> list[str]:
        """Value to name, for everything the header gives numbers to.

        A tag, an error code, and a flags bit are all just integers at the
        boundary. These tables are what turn one back into something a caller
        can put in a log without a lookup table of their own.
        """
        lines: list[str] = ["/** Tag value to arm name, for every tagged union. */", "export const KINDS = Object.freeze({"]
        for name in sorted(self.abi.unions):
            arms = self.abi.union_arms(self.abi.unions[name])
            if not arms:
                continue
            lines.append("  " + name + ": Object.freeze({")
            for value, variant, _body in arms:
                lines.append("    " + str(value) + ": " + quote(variant) + ",")
            lines.append("  }),")
        lines.append("});")
        lines.append("")

        for enum_name in sorted(self.abi.enums):
            enum = self.abi.enums[enum_name]
            if enum_name.endswith("_Tag"):
                continue
            lines.append("/** " + (self.one_line(enum.doc) or ("The " + enum_name + " codes.")) + " */")
            lines.append("export const " + enum_name + " = Object.freeze({")
            for value, (variant, _explicit) in enumerate(enum.variants):
                lines.append("  " + variant + ": " + str(value) + ",")
            lines.append("});")
            lines.append("")

        lines.extend(self.tag_tables())

        codes = self.code_family()
        if codes:
            lines.append("/** Why a document was refused. Zero means it was not refused. */")
            lines.append("export const LoadError = Object.freeze({")
            for value, variant in codes:
                lines.append("  " + str(value) + ": " + quote(variant) + ",")
            lines.append("});")
            lines.append("")
        return lines

    def code_family(self) -> list[tuple[int, str]]:
        """The constants that are the codes of the integer alias named `LoadError`.

        `cbindgen` writes a Rust `pub type LoadError = i32` as a typedef and its
        codes as constants beside it, with nothing joining the two up. The join
        available here is the naming: a constant is one of the codes when the
        word after the shared prefix is a word the alias's own name spells out.
        """
        import os

        scalars = [c for c in self.abi.consts if isinstance(c.type, Scalar)]
        if len(scalars) < 2:
            return []
        shared = os.path.commonprefix([c.name for c in scalars])
        if not shared:
            return []

        letters = set(ch.lower() for ch in "LoadError" if ch.isalpha())
        family = []
        for const in scalars:
            rest = const.name[len(shared) :]
            head = rest.split("_")[0].lower()
            if head and all(ch in letters for ch in head):
                family.append((const, rest))
        if len(family) < 2:
            return []

        prefix = os.path.commonprefix([rest for _c, rest in family])
        out = [(int(c.value), self.title(c.name[len(shared) + len(prefix) :])) for c, _rest in family]
        return sorted(out)

    def tag_tables(self) -> list[str]:
        """Tag name to tag value, the other way round from `KINDS`.

        Building one needs the value and reading one needs the name, so both
        directions are worth having. They come from the tag enums the header
        declares, so an arm added in Rust shows up in both without anything
        here being told.
        """
        lines = ["/** Tag name to tag value, for every tagged union. */", "export const TAGS = Object.freeze({"]
        for name in sorted(self.abi.enums):
            enum = self.abi.enums[name]
            if not name.endswith("_Tag"):
                continue
            lines.append("  " + name + ": Object.freeze({")
            for value, (variant, _explicit) in enumerate(enum.variants):
                lines.append("    " + variant + ": " + str(value) + ",")
            lines.append("  }),")
        lines.append("});")
        lines.append("")
        return lines

    def title(self, text: str) -> str:
        return "".join(word[:1].upper() + word[1:].lower() for word in text.split("_") if word)

    def constants_js(self) -> list[str]:
        lines = ["/** The constants the header declares, under the names it gives them. */", "export const CONSTANTS = Object.freeze({"]
        for const in self.abi.consts:
            value = str(int(const.value)) if isinstance(const.value, int) else str(const.value)
            lines.append("  " + const.name + ": " + value + ",")
        lines.append("});")
        lines.append("")
        for const in self.abi.consts:
            lines.append("export const " + const.name + " = CONSTANTS." + const.name + ";")
        lines.append("")
        return lines

    def raw_class(self) -> list[str]:
        lines = [
            "/** Every entry point, called the way the module says it is. */",
            "export class Raw {",
            "  constructor(instance) {",
            "    this.exports = instance.exports;",
            "    this.memory = instance.exports.memory;",
            "  }",
            "",
            "  /** A view over linear memory, current as of this call. */",
            "  bytes() {",
            "    return view(this.memory);",
            "  }",
            "",
        ]
        for func in self.ordered_functions():
            lines.extend(self.raw_method(func))
        lines.append("}")
        lines.append("")
        return lines

    def raw_method(self, func: Func) -> list[str]:
        signature = ", ".join(camel(p.name) for p in func.params)
        lines = ["  " + func.name + "(" + signature + ") {"]

        wasm = func.wasm
        returns_struct = wasm is not None and wasm.sret is not None

        arguments: list[str] = []
        if returns_struct:
            lines.append("    const slot = reserve(this.memory, " + str(self.layout.size(func.ret)) + ");")
            arguments.append("slot")

        needs_view = any(wasm is not None and index in wasm.by_value for index in range(len(func.params)))
        if needs_view:
            # A view taken before the reserve below would be detached by it, so
            # one is taken after each reserve rather than once.
            lines.append("    let view;")

        for index, param in enumerate(func.params):
            name = camel(param.name)
            if wasm is not None and index in wasm.by_value:
                # An aggregate passed by value is copied into linear memory by
                # the caller, so it arrives as a pointer to a copy.
                lines.append("    const " + name + "At = reserve(this.memory, " + str(self.layout.size(param.type)) + ");")
                lines.append("    view = this.bytes();")
                source = name + ".bits" if self.is_bitflag(param) else name
                lines.append(self.write_expr(param.type, name + "At", source, indent="    "))
                arguments.append(name + "At")
            else:
                arguments.append(name)

        call = "this.exports." + func.name + "(" + ", ".join(arguments) + ")"
        if returns_struct:
            lines.append("    " + call + ";")
            lines.append("    return take(this.memory, slot, " + str(self.layout.size(func.ret)) + ");")
        elif self.is_void(func.ret):
            lines.append("    " + call + ";")
        else:
            lines.append("    return " + call + ";")
        lines.append("  }")
        lines.append("")
        return lines

    def is_bitflag(self, param) -> bool:
        return isinstance(param.type, Named) and param.type.name in self.abi.bitflags

    def is_void(self, type: Type) -> bool:
        resolved = self.abi.resolve(type)
        return isinstance(resolved, Scalar) and resolved.name == "void"

    # ------------------------------------------------------------------
    # The convenience layer
    # ------------------------------------------------------------------

    def render_index(self) -> list[str]:
        lines: list[str] = []
        lines.extend(self.banner())
        lines.extend(
            [
                "/**",
                " * The Louder engine, from JavaScript.",
                " *",
                " * Names are strings, documents are strings, a variant is a number or a",
                " * string, and an event carries a kind and a decoded body. The engine has no",
                " * opinion about any of that; this layer is where it is expressed.",
                " *",
                " *     const engine = await Louder.load(louderWasm);",
                " *     engine.loadConfig(document);",
                " *     engine.loadConversation('chat', document);",
                " *",
                " *     const actor = engine.addActor('chat', 0, { speakAs: ['A'] });",
                " *     engine.choose('chat', actor, 0);",
                " *     for (const event of engine.drainEvents()) {",
                " *       console.log(event.kind, event.value);",
                " *     }",
                " */",
                "",
                'import {',
                '  CONSTANTS,',
                '  KINDS,',
                '  LoadError,',
                '  LouderError,',
                '  Raw,',
                '  TAGS,',
                '  instantiate,',
                '  load,',
                '  readEventData,',
                '  readLouderAsk,',
                '  readLouderVariant,',
                '} from "./raw.js";',
                'import { fromBytes, readInto, reserve, view, withBytes } from "./memory.js";',
                "",
                "export { Raw, instantiate, load, KINDS, TAGS, CONSTANTS, LoadError, LouderError };",
                "",
                "/** The id of nothing, which is what a failed lookup comes back as. */",
                "export const NAME_NONE = " + str(self.const_value("LOUDER_NAME_NONE")) + ";",
                "",
                "/** The id of no actor, which is what an actor outside the flow reads as. */",
                "export const ACTOR_NONE = " + str(self.const_value("LOUDER_ACTOR_NONE")) + ";",
                "",
                "/** The position of no hub, and where a cursor over hubs starts. */",
                "export const BLANK_POSITION = -1;",
                "",
                "",
            ]
        )
        lines.extend(self.HELPERS.strip("\n").splitlines())
        lines.append("")
        lines.extend(self.render_class())
        return lines

    def const_value(self, name: str) -> int:
        for const in self.abi.consts:
            if const.name == name:
                return int(const.value)
        return -1

    HELPERS = '''
function flags(value) {
  return typeof value === "number" ? { bits: value } : (value ?? { bits: 0 });
}
'''

    def iterators(self) -> list[Func]:
        found = []
        for func in self.abi.funcs:
            if not func.name.startswith("next_"):
                continue
            start = [p for p in func.params if p.name == "start"]
            if not start or not isinstance(start[0].type, Named):
                continue
            if not isinstance(func.ret, Named) or func.ret.name != start[0].type.name:
                continue
            if func.ret.name not in SENTINELS:
                continue
            found.append(func)
        return found

    def render_class(self) -> list[str]:
        lines = [
            "/** One engine context: the name table, the conversations, and the queue. */",
            "export class Louder {",
            "  constructor(raw) {",
            "    this.raw = raw;",
            "    this.ctx = raw.init_louder();",
            "    this.closed = false;",
            "  }",
            "",
            "  /** Load an engine module and open a context on it. */",
            "  static async load(source) {",
            "    return new Louder(await load(source));",
            "  }",
            "",
            "  static async instantiate(source) {",
            "    return new Louder(await instantiate(source));",
            "  }",
            "",
            "  close() {",
            "    if (!this.closed) {",
            "      this.closed = true;",
            "      this.raw.deinit_louder(this.ctx);",
            "    }",
            "  }",
            "",
        ]
        for section in (
            self.js_names(),
            self.js_documents(),
            self.js_hubs(),
            self.js_actors(),
            self.js_state(),
            self.js_iterators(),
            self.js_events(),
            self.js_asks(),
            self.js_raw(),
        ):
            lines.extend(section)
        lines.append("}")
        return lines

    def js_names(self) -> list[str]:
        return [
            "  /** The id of some text, or NAME_NONE if it was never declared. */",
            "  lookup(text) {",
            "    const [pointer, length] = withBytes(this.raw.memory, text);",
            "    return this.raw.get_named_id(this.ctx, pointer, length);",
            "  }",
            "",
            "  /** The id of some text, declaring it the first time. An id passes through. */",
            "  declare(text) {",
            "    if (typeof text === \"number\") {",
            "      return text;",
            "    }",
            "    const existing = this.lookup(text);",
            "    if (existing !== NAME_NONE) {",
            "      return existing;",
            "    }",
            "    const [pointer, length] = withBytes(this.raw.memory, text);",
            "    return this.raw.declare_name(this.ctx, pointer, length);",
            "  }",
            "",
            "  /** An alias for declare, for when the text really is a name. */",
            "  name(text) {",
            "    return this.declare(text);",
            "  }",
            "",
            "  /** The text behind an id, decoded as UTF-8. */",
            "  text(id) {",
            "    return fromBytes(this.#read(this.raw.read_name.bind(this.raw), [this.ctx, Number(id)]));",
            "  }",
            "",
            "  /** The text behind an id, or null when the id names nothing. */",
            "  optionalText(id) {",
            "    return Number(id) === NAME_NONE ? null : this.text(id);",
            "  }",
            "",
        ]

    def js_documents(self) -> list[str]:
        return [
            "  /** Load a conversation from a JSON document. */",
            "  loadConversation(conversation, document) {",
            "    const name = this.declare(conversation);",
            "    const [pointer, length] = withBytes(this.raw.memory, document);",
            "    return LoadError[this.raw.load_conversation_json(this.ctx, name, pointer, length)];",
            "  }",
            "",
            "  /** Load the speaker config, which is a singleton. */",
            "  loadConfig(document) {",
            "    const [pointer, length] = withBytes(this.raw.memory, document);",
            "    return LoadError[this.raw.load_conversation_config(this.ctx, pointer, length)];",
            "  }",
            "",
            "  /** Remove every speaker. Conversations are left alone. */",
            "  clearConfig() {",
            "    return this.raw.clear_config(this.ctx) === 0;",
            "  }",
            "",
            "  /** Remove a conversation and the actors in it. */",
            "  removeConversation(conversation) {",
            "    return this.raw.remove_conversation(this.ctx, this.declare(conversation)) === 0;",
            "  }",
            "",
        ]

    def js_hubs(self) -> list[str]:
        text_calls = [
            ("hubText", "get_hub_text", "The text of a hub."),
            ("hubSpeaker", "get_hub_speaker", "The speaker of a hub."),
            ("hubChannel", "get_hub_channel", "The channel of a hub."),
            ("hubExpressions", "get_hub_expressions", "The expressions of a hub, which the schema keeps newline separated."),
            ("hubCallbacks", "get_hub_callbacks", "The callbacks of a hub."),
        ]
        out: list[str] = []
        for method, entry, summary in text_calls:
            out.extend(
                [
                    "  /** " + summary + " */",
                    "  " + method + "(conversation, position) {",
                    "    return this.text(this.raw." + entry + "(this.ctx, this.declare(conversation), position));",
                    "  }",
                    "",
                ]
            )
        out.extend(
            [
                "  /** The priority of a hub. */",
                "  hubPriority(conversation, position) {",
                "    return this.raw.get_hub_priority(this.ctx, this.declare(conversation), position);",
                "  }",
                "",
                "  /** The flags on a hub, as bits. */",
                "  hubFlags(conversation, position) {",
                "    return this.raw.get_hub_flags(this.ctx, this.declare(conversation), position);",
                "  }",
                "",
                "  /** Additional data on a hub, under a key. */",
                "  hubAdditional(conversation, position, key) {",
                "    const bytes = this.raw.get_hub_additional(this.ctx, this.declare(conversation), position, this.declare(key));",
                "    return this.#variant(bytes);",
                "  }",
                "",
                "  /** How many choices are available to an actor on a hub. */",
                "  hubChoiceCount(conversation, position, actor) {",
                "    return this.raw.get_hub_choice_count(this.ctx, this.declare(conversation), position, actor);",
                "  }",
                "",
                "  /** The text of one choice, which is the text of the hub it leads to. */",
                "  hubChoiceText(conversation, position, actor, index) {",
                "    return this.text(this.raw.get_hub_choice_text(this.ctx, this.declare(conversation), position, actor, index));",
                "  }",
                "",
                "  /** Where one choice leads. */",
                "  hubChoiceLink(conversation, position, actor, index) {",
                "    return this.raw.get_hub_choice_link(this.ctx, this.declare(conversation), position, actor, index);",
                "  }",
                "",
            ]
        )
        return out

    def js_actors(self) -> list[str]:
        return [
            "  /** Put an actor into a conversation and return its id. */",
            "  addActor(conversation, position, options = {}) {",
            "    const speakAs = this.#nameList(options.speakAs);",
            "    const listenTo = this.#nameList(options.listenTo);",
            "    return this.raw.add_actor(",
            "      this.ctx,",
            "      this.declare(conversation),",
            "      position,",
            "      flags(options.flags),",
            "      speakAs.pointer,",
            "      speakAs.count,",
            "      listenTo.pointer,",
            "      listenTo.count,",
            "    );",
            "  }",
            "",
            "  /** Take an actor out of a conversation. */",
            "  removeActor(conversation, actor) {",
            "    return this.raw.remove_actor(this.ctx, this.declare(conversation), actor) === 0;",
            "  }",
            "",
            "  /** Where an actor is standing. */",
            "  actorPosition(conversation, actor) {",
            "    return this.raw.get_actor_position(this.ctx, this.declare(conversation), actor);",
            "  }",
            "",
            "  /** Which channel an actor is on. */",
            "  actorChannel(conversation, actor) {",
            "    return this.text(this.raw.get_actor_channel(this.ctx, this.declare(conversation), actor));",
            "  }",
            "",
            "  /** The flags on an actor, as bits. */",
            "  actorFlags(conversation, actor) {",
            "    return this.raw.get_actor_flags(this.ctx, this.declare(conversation), actor);",
            "  }",
            "",
            "  /** The priority on an actor. */",
            "  actorPriority(conversation, actor) {",
            "    return this.raw.get_actor_priority(this.ctx, this.declare(conversation), actor);",
            "  }",
            "",
            "  /** Additional data on an actor, under a key. */",
            "  actorAdditional(conversation, actor, key) {",
            "    const bytes = this.raw.get_actor_additional(this.ctx, this.declare(conversation), actor, this.declare(key));",
            "    return this.#variant(bytes);",
            "  }",
            "",
            "  /** Take a choice on the hub an actor is standing on. */",
            "  choose(conversation, actor, index) {",
            "    return this.raw.select_choice(this.ctx, this.declare(conversation), actor, index) === 0;",
            "  }",
            "",
            "  /** Move an actor as though it had walked there. */",
            "  warp(conversation, actor, target, options = {}) {",
            "    const channel = options.channel === undefined ? NAME_NONE : this.declare(options.channel);",
            "    return this.raw.force_warp(this.ctx, this.declare(conversation), actor, target, channel, options.priority ?? 0) === 0;",
            "  }",
            "",
            "  /** Move an actor straight there. */",
            "  teleport(conversation, actor, target, options = {}) {",
            "    const channel = options.channel === undefined ? NAME_NONE : this.declare(options.channel);",
            "    return this.raw.force_position(this.ctx, this.declare(conversation), actor, target, channel, options.priority ?? 0) === 0;",
            "  }",
            "",
            "  /** A list of names as the array of ids the engine reads. */",
            "  #nameList(values) {",
            "    const list = values ?? [];",
            "    const pointer = reserve(this.raw.memory, Math.max(list.length, 1) * 4);",
            "    const bytes = view(this.raw.memory);",
            "    for (let index = 0; index < list.length; index += 1) {",
            "      bytes.setInt32(pointer + index * 4, this.declare(list[index]), true);",
            "    }",
            "    return { pointer, count: list.length };",
            "  }",
            "",
        ]

    def js_state(self) -> list[str]:
        return [
            "  /** A story state value, or a zero variant if it was never written. */",
            "  get(conversation, key) {",
            "    return this.#variant(this.raw.get_story_state(this.ctx, this.declare(conversation), this.declare(key)));",
            "  }",
            "",
            "  /** Write a story state value. */",
            "  set(conversation, key, value) {",
            "    this.raw.set_story_state(",
            "      this.ctx,",
            "      this.declare(conversation),",
            "      this.declare(key),",
            "      this.#makeVariant(value),",
            "    );",
            "  }",
            "",
            "  /** The whole story state as a JSON document. */",
            "  stateJson(conversation) {",
            "    return fromBytes(this.#read(this.raw.read_state_json.bind(this.raw), [this.ctx, this.declare(conversation)], 1 << 16));",
            "  }",
            "",
            "  /** The story state and every actor, as a JSON document. */",
            "  snapshotJson(conversation) {",
            "    return fromBytes(this.#read(this.raw.read_snapshot_json.bind(this.raw), [this.ctx, this.declare(conversation)], 1 << 16));",
            "  }",
            "",
            "  #makeVariant(value) {",
            "    const tag = TAGS.LouderVariant_Tag;",
            "    if (typeof value === \"boolean\") {",
            "      return { tag: tag.Int, int: value ? 1 : 0 };",
            "    }",
            "    if (typeof value === \"number\") {",
            "      return Number.isInteger(value)",
            "        ? { tag: tag.Int, int: value }",
            "        : { tag: tag.Float, float: value };",
            "    }",
            "    return { tag: tag.String, string: this.declare(value) };",
            "  }",
            "",
            "  #variant(bytes) {",
            "    const variant = readLouderVariant(new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength), 0);",
            "    const kind = KINDS.LouderVariant[variant.tag];",
            "    if (kind === \"Int\") {",
            "      return variant.int;",
            "    }",
            "    if (kind === \"Float\") {",
            "      return variant.float;",
            "    }",
            "    return this.text(variant.string);",
            "  }",
            "",
            "  /** Call something that fills a buffer, growing until the whole answer fits. */",
            "  #read(call, args, size) {",
            "    return readInto(this.raw.memory, call, args, size);",
            "  }",
            "",
        ]

    def js_iterators(self) -> list[str]:
        out = [
            "  /**",
            "   * Every one of these is a cursor in the C header: a `start`, the thing it",
            "   * returns, and a sentinel for the end of the walk. Here they are",
            "   * generators, so the walk reads as a `for...of` and the caller cannot",
            "   * forget to advance.",
            "   */",
        ]
        for func in self.iterators():
            end = SENTINELS[func.ret.name]
            args = [p for p in func.params if p.name not in ("start", "ctx")]
            signature = ", ".join(camel(p.name) for p in args)
            call = ", ".join(["this.ctx"] + [self.js_argument(p) for p in args])
            out.extend(
                [
                    "  *" + self.iterator_name(func) + "(" + signature + ") {",
                    "    let start = " + end + ";",
                    "    for (;;) {",
                    "      const found = this.raw." + func.name + "(" + call + ", start);",
                    "      if (found === " + end + ") {",
                    "        return;",
                    "      }",
                    "      yield found;",
                    "      start = found;",
                    "    }",
                    "  }",
                    "",
                ]
            )
        return out

    def js_argument(self, param) -> str:
        name = camel(param.name)
        if isinstance(param.type, Named) and param.name in NAME_ARGUMENTS:
            return "this.declare(" + name + ")"
        return name

    def iterator_name(self, func: Func) -> str:
        """`next_hub_by_channel` is a cursor; here it is `hubByChannel`.

        The `next_` and the cursor shape both belong to the C boundary, and
        neither survives the crossing: what is left is a thing you iterate.
        """
        return camel(func.name[len("next_") :])

    def option_body_offset(self, func_name: str) -> int:
        """Where the payload of an option sits inside the option.

        `get_event` hands back a tagged union whose arm is an event, so the
        event's own fields start after the tag rather than at zero. That offset
        comes from the declaration, so a change to the shape moves it.
        """
        if not self.abi.has_func(func_name):
            return 0
        ret = self.abi.func(func_name).ret
        if not isinstance(ret, Named):
            return 0
        union = self.abi.unions.get(ret.name)
        if union is None or not union.variants:
            return 0
        return self.layout.variant_offset(union, union.variants[0])

    def js_events(self) -> list[str]:
        offsets = self.offsets()["LouderEvent"]
        base = self.option_body_offset("get_event")
        out = [
            "  /** Every queued event, oldest first, leaving the queue alone. */",
            "  *events() {",
            "    for (let index = 0; ; index += 1) {",
            "      const bytes = this.raw.get_event(this.ctx, index);",
            "      const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);",
            "      if (view.getInt32(0, true) !== 1) {",
            "        return;",
            "      }",
            "      yield this.#event(view);",
            "    }",
            "  }",
            "",
            "  /** Every queued event, oldest first, and empty the queue. */",
            "  drainEvents() {",
            "    const out = [...this.events()];",
            "    this.raw.flush_events(this.ctx);",
            "    return out;",
            "  }",
            "",
            "  #event(view) {",
            "    const data = readEventData(view, " + str(base + offsets["data"]) + ");",
            "    const kind = KINDS.EventData[data.tag] ?? String(data.tag);",
            "    return {",
            "      conversation: this.text(view.getInt32(" + str(base + offsets["conversation"]) + ", true)),",
            "      channel: this.text(view.getInt32(" + str(base + offsets["channel"]) + ", true)),",
            "      actor: view.getInt32(" + str(base + offsets["actor"]) + ", true),",
            "      kind,",
            "      value: this.#eventValue(kind, data),",
            "    };",
            "  }",
            "",
            "  /** The arm the tag names, already decoded. */",
            "  #eventValue(kind, data) {",
            "    switch (kind) {",
        ]
        arms = self.abi.unions.get("EventData")
        if arms is not None:
            for _value, name, carrier in self.abi.union_arms(arms):
                out.append("      case " + quote(name) + ":")
                out.append("        return undefined;" if carrier is None else "        return data." + camel(snake(name)) + ";")
        out.extend(
            [
                "      default:",
                "        return undefined;",
                "    }",
                "  }",
                "",
            ]
        )
        return out

    def js_asks(self) -> list[str]:
        ask = self.offsets()["LouderAsk"]
        base = self.option_body_offset("get_ask")
        return [
            "  /** Every pending ask. The order is not stable. */",
            "  *asks() {",
            "    for (let index = 0; ; index += 1) {",
            "      const bytes = this.raw.get_ask(this.ctx, index);",
            "      const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);",
            "      if (view.getInt32(0, true) !== 1) {",
            "        return;",
            "      }",
            "      const ask = readLouderAsk(view, " + str(base + ask["conversation"]) + ");",
            "      yield {",
            "        conversation: this.text(ask.conversation),",
            "        channel: this.text(ask.channel),",
            "        actor: ask.actor,",
            "        ask: this.text(ask.ask),",
            "      };",
            "    }",
            "  }",
            "",
            "  /** Answer the ask at an index. Answering shifts the ones after it. */",
            "  answer(index, verdict) {",
            "    return this.raw.answer_ask(this.ctx, index, verdict) === 0;",
            "  }",
            "",
        ]

    def js_raw(self) -> list[str]:
        return [
            "  /** Call any entry point by name, with the signatures of the raw layer. */",
            "  call(name, ...args) {",
            "    return this.raw[name](...args);",
            "  }",
            "",
            "  /** The dialogue rules in effect. */",
            "  rules() {",
            "    return this.raw.get_dialogue_rules(this.ctx);",
            "  }",
            "",
            "  /** Replace the dialogue rules. They take effect on the next move. */",
            "  setRules(rules) {",
            "    return this.raw.set_dialogue_rules(this.ctx, flags(rules)) === 0;",
            "  }",
        ]

    # ------------------------------------------------------------------
    # TypeScript
    # ------------------------------------------------------------------

    def render_types(self) -> list[str]:
        lines: list[str] = []
        lines.extend(self.banner())
        lines.append("")
        lines.append("export { Raw, KINDS, CONSTANTS } from \"./raw.js\";")
        lines.append('import type { Raw } from "./raw.js";')
        lines.append("")
        lines.append("/** One queued event, with the names in it already decoded. */")
        lines.append("export interface Event {")
        lines.append("  conversation: string;")
        lines.append("  channel: string;")
        lines.append("  actor: number;")
        lines.append("  kind: string;")
        lines.append("  value: unknown;")
        lines.append("}")
        lines.append("")
        lines.append("/** A question the engine has asked and is waiting on. */")
        lines.append("export interface Ask {")
        lines.append("  conversation: string;")
        lines.append("  channel: string;")
        lines.append("  actor: number;")
        lines.append("  ask: string;")
        lines.append("}")
        lines.append("")
        lines.append("/** Options for putting an actor into a conversation. */")
        lines.append("export interface ActorOptions {")
        lines.append("  speakAs?: Array<string | number>;")
        lines.append("  listenTo?: Array<string | number>;")
        lines.append("  flags?: number;")
        lines.append("}")
        lines.append("")
        lines.append("/** Options for moving an actor where it could not otherwise reach. */")
        lines.append("export interface MoveOptions {")
        lines.append("  channel?: string | number;")
        lines.append("  priority?: number;")
        lines.append("}")
        lines.append("")
        lines.append("export type LoadError = string;")
        lines.append("")
        lines.append("export declare const NAME_NONE: number;")
        lines.append("export declare const ACTOR_NONE: number;")
        lines.append("export declare const BLANK_POSITION: number;")
        lines.append("")
        lines.extend(self.ts_structs())
        lines.extend(self.ts_unions())
        lines.extend(self.ts_class())
        return lines

    def ts_structs(self) -> list[str]:
        lines: list[str] = []
        for name in sorted(self.abi.structs):
            if name in self.abi.union_bodies:
                continue
            struct = self.abi.structs[name]
            lines.append("/** " + (self.one_line(struct.doc) or ("The " + name + " type.")) + " */")
            lines.append("export interface " + name + " {")
            for placed in self.layout.fields(struct):
                lines.append("  " + camel(placed.name) + ": " + self.ts_type(placed.type) + ";")
            lines.append("}")
            lines.append("")
        return lines

    def ts_unions(self) -> list[str]:
        lines: list[str] = []
        for name in sorted(self.abi.unions):
            union = self.abi.unions[name]
            arms = self.abi.union_arms(union)
            if not arms:
                continue
            lines.append("/** " + (self.one_line(union.doc) or ("The " + name + " type.")) + " */")
            lines.append("export interface " + name + " {")
            lines.append("  tag: number;")
            for _value, variant, carrier in arms:
                if carrier is None:
                    continue
                lines.append("  " + camel(snake(variant)) + "?: " + self.ts_type(carrier.type) + ";")
            lines.append("}")
            lines.append("")
        return lines

    def ts_type(self, type: Type) -> str:
        if isinstance(type, Pointer):
            return "number"
        if isinstance(type, Scalar):
            return _JS_SCALARS.get(type.name, "number")
        resolved = self.abi.resolve(type)
        if isinstance(resolved, Scalar):
            return _JS_SCALARS.get(resolved.name, "number")
        return self.underlying(type)

    def ts_class(self) -> list[str]:
        lines = [
            "/** One engine context: the name table, the conversations, and the queue. */",
            "export declare class Louder {",
            "  constructor(raw: Raw);",
            "  readonly raw: Raw;",
            "  readonly ctx: number;",
            "  readonly closed: boolean;",
            "  close(): void;",
            "  static load(source: string | URL | ArrayBuffer | Response): Promise<Louder>;",
            "  static instantiate(source: ArrayBuffer | Response): Promise<Louder>;",
            "  lookup(text: string): number;",
            "  declare(text: string | number): number;",
            "  name(text: string | number): number;",
            "  text(id: number): string;",
            "  optionalText(id: number): string | null;",
            "  loadConversation(conversation: string, document: string): LoadError;",
            "  loadConfig(document: string): LoadError;",
            "  clearConfig(): boolean;",
            "  removeConversation(conversation: string): boolean;",
            "  hubText(conversation: string, position: number): string;",
            "  hubSpeaker(conversation: string, position: number): string;",
            "  hubChannel(conversation: string, position: number): string;",
            "  hubExpressions(conversation: string, position: number): string;",
            "  hubCallbacks(conversation: string, position: number): string;",
            "  hubPriority(conversation: string, position: number): number;",
            "  hubFlags(conversation: string, position: number): number;",
            "  hubAdditional(conversation: string, position: number, key: string): number | string;",
            "  hubChoiceCount(conversation: string, position: number, actor: number): number;",
            "  hubChoiceText(conversation: string, position: number, actor: number, index: number): string;",
            "  hubChoiceLink(conversation: string, position: number, actor: number, index: number): number;",
            "  addActor(conversation: string, position: number, options?: ActorOptions): number;",
            "  removeActor(conversation: string, actor: number): boolean;",
            "  actorPosition(conversation: string, actor: number): number;",
            "  actorChannel(conversation: string, actor: number): string;",
            "  actorFlags(conversation: string, actor: number): number;",
            "  actorPriority(conversation: string, actor: number): number;",
            "  actorAdditional(conversation: string, actor: number, key: string): number | string;",
            "  choose(conversation: string, actor: number, index: number): boolean;",
            "  warp(conversation: string, actor: number, target: number, options?: MoveOptions): boolean;",
            "  teleport(conversation: string, actor: number, target: number, options?: MoveOptions): boolean;",
            "  get(conversation: string, key: string): number | string;",
            "  set(conversation: string, key: string, value: number | string | boolean): void;",
            "  stateJson(conversation: string): string;",
            "  snapshotJson(conversation: string): string;",
            "  events(): Generator<Event>;",
            "  drainEvents(): Event[];",
            "  asks(): Generator<Ask>;",
            "  answer(index: number, verdict: number): boolean;",
            "  call(name: string, ...args: unknown[]): unknown;",
            "  rules(): number;",
            "  setRules(rules: number): boolean;",
        ]
        for func in self.iterators():
            args = [p for p in func.params if p.name not in ("start", "ctx")]
            signature = ", ".join(
                camel(p.name) + ": " + ("string | number" if p.name in NAME_ARGUMENTS else "number") for p in args
            )
            lines.append("  *" + self.iterator_name(func) + "(" + signature + "): Generator<number>;")
        lines.append("}")
        lines.append("")
        return lines

    def render_raw_types(self) -> list[str]:
        lines: list[str] = []
        lines.extend(self.banner())
        lines.append("")
        lines.append("/** How much of linear memory each declared type occupies. */")
        lines.append("export declare const SIZE: Readonly<Record<string, number>>;")
        lines.append("")
        lines.append("/** Where each field sits inside its type. */")
        lines.append("export declare const OFFSET: Readonly<Record<string, Readonly<Record<string, number>>>>;")
        lines.append("")
        lines.append("/** Tag value to arm name, for every tagged union. */")
        lines.append("export declare const KINDS: Readonly<Record<string, Readonly<Record<number, string>>>>;")
        lines.append("")
        lines.append("/** The constants the header declares. */")
        lines.append("export declare const CONSTANTS: Readonly<Record<string, number>>;")
        lines.append("")
        lines.append("/** Why a document was refused. Zero means it was not refused. */")
        lines.append("export declare const LoadError: Readonly<Record<number, string>>;")
        lines.append("")
        lines.append("/** The raw layer: the module's exports, called with its own conventions. */")
        lines.append("export declare class Raw {")
        lines.append("  constructor(instance: WebAssembly.Instance);")
        lines.append("  readonly exports: WebAssembly.Exports;")
        lines.append("  readonly memory: WebAssembly.Memory;")
        lines.append("  bytes(): DataView;")
        for func in self.ordered_functions():
            arguments = ", ".join(camel(p.name) + ": " + self.ts_type(p.type) for p in func.params)
            lines.append("  " + func.name + "(" + arguments + "): " + self.raw_return(func) + ";")
        lines.append("}")
        lines.append("")
        lines.append("export declare function instantiate(source: ArrayBuffer | Response): Promise<Raw>;")
        lines.append("export declare function load(source: string | URL | ArrayBuffer | Response): Promise<Raw>;")
        lines.append("")
        for struct in self.ordered_structs():
            lines.append("export declare function read" + struct.name + "(view: DataView, at: number): unknown;")
        for union in self.ordered_unions():
            lines.append("export declare function read" + union.name + "(view: DataView, at: number): unknown;")
        return lines

    def raw_return(self, func: Func) -> str:
        if func.wasm is not None and func.wasm.sret is not None:
            return "Uint8Array"
        if self.is_void(func.ret):
            return "void"
        return self.ts_type(func.ret)

    # ------------------------------------------------------------------

    def render_package(self) -> list[str]:
        return [
            "{",
            '  "name": "louder",',
            '  "version": "0.1.0",',
            '  "description": "Bindings for the Louder dialogue engine",',
            '  "type": "module",',
            '  "main": "./louder/index.js",',
            '  "types": "./louder/index.d.ts",',
            '  "exports": {',
            '    ".": {',
            '      "types": "./louder/index.d.ts",',
            '      "default": "./louder/index.js"',
            "    }",
            '  },',
            '  "files": ["louder"],',
            '  "license": "MIT"',
            "}",
        ]

    def render_readme(self) -> list[str]:
        return (
            [
                "# Louder, for JavaScript",
                "",
                "Generated from the C header of the `louder_rust` engine. Do not edit; run",
                "`python engine/bindings/generate.py` and this directory comes back.",
                "",
                "There is no glue code. The engine is built to WebAssembly, the module is",
                "instantiated directly, and its exports are called with the arguments its own",
                "type section says they take. The same module runs in Node, in Deno and in a",
                "browser.",
                "",
                "```js",
                "import { readFile } from 'node:fs/promises';",
                "import { Louder } from './louder/index.js';",
                "",
                "const engine = await Louder.load(await readFile('louder_rust.wasm'));",
                "",
                "engine.loadConfig(await readFile('speakers.json', 'utf8'));",
                "engine.loadConversation('chat', await readFile('chat.json', 'utf8'));",
                "",
                "const actor = engine.addActor('chat', 0, { speakAs: ['A'] });",
                "engine.choose('chat', actor, 0);",
                "",
                "for (const event of engine.drainEvents()) {",
                "  console.log(event.kind, event.value);",
                "}",
                "",
                "for (const hub of engine.hubByChannel('chat', 'main')) {",
                "  console.log(hub, engine.hubText('chat', hub));",
                "}",
                "",
                "console.log(engine.get('chat', 'mood'));",
                "engine.close();",
                "```",
                "",
                "## Building the engine for WebAssembly",
                "",
                "```",
                "cargo build -p louder_rust --release --target wasm32-unknown-unknown",
                "```",
                "",
                "The module lands under `engine/target/wasm32-unknown-unknown/` and is copied",
                "beside these files.",
                "",
                "## The layers",
                "",
                "- `louder/raw.js` is the boundary as the module exports it: `SIZE`,",
                "  `OFFSET`, a codec per declared type, and one method per entry point.",
                "- `louder/index.js` is the same engine in JavaScript terms.",
                "- `louder/index.d.ts` is the same in TypeScript terms, and is the",
                "  TypeScript binding.",
                "- `louder/memory.js` owns the scratch memory that both shapes the C ABI",
                "  takes on WebAssembly need. It grows the module's memory and takes the",
                "  pages that come back, which the engine's own allocator never sees.",
                "",
                "## What was generated",
                "",
                "- " + self.counts(),
                "",
            ]
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
            + " enums and "
            + str(shapes["constants"])
            + " constants"
        )


def quote(text: str) -> str:
    """A JavaScript string literal, for a name the header declares."""
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def _unused(*_: object) -> None:
    return None


_unused(screaming)
