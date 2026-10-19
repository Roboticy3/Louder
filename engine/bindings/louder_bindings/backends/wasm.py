"""WebAssembly, which is the one target that needs no glue code at all.

The engine compiled to wasm32 exports its C boundary directly, so a host loads
the module and calls the exports. Nothing is generated in between: what a host
needs is the module's own signature for every export, which is not what a C
header says, because the LLVM WebAssembly ABI passes aggregates differently.

So this backend writes three things, all from the module and the header together:

- `louder_wasm.h`, a header that declares every export with the signature it
  actually has, with the size of every type pinned down by a static assertion,
  so a compiler refuses to build it against the wrong layout;
- `louder_wasm.json`, the same information for a host that is not C;
- the module itself, so the directory is self-contained.

The signature is read out of the module by `wasm.py` rather than worked out from
the rules, and a disagreement raises before anything is written. That is the
whole point of this backend: a host that trusts these signatures is trusting the
binary.
"""

from __future__ import annotations

import json
from typing import Optional

from ..ir import Enum, Func, Pointer, Scalar, Type
from .base import Emitted
from .clike import ClikeBackend

# What each scalar is in the module itself. A pointer and a `size_t` are both
# `i32` on wasm32; nothing else narrows.
_WASM_OF_SCALAR = {
    "void": None,
    "bool": "i32",
    "int8_t": "i32",
    "uint8_t": "i32",
    "char": "i32",
    "int16_t": "i32",
    "uint16_t": "i32",
    "short": "i32",
    "int32_t": "i32",
    "uint32_t": "i32",
    "int": "i32",
    "unsigned": "i32",
    "float": "f32",
    "int64_t": "i64",
    "uint64_t": "i64",
    "long long": "i64",
    "double": "f64",
    "size_t": "i32",
    "ptrdiff_t": "i32",
    "intptr_t": "i32",
    "uintptr_t": "i32",
}


class WasmBackend(ClikeBackend):
    name = "wasm"
    label = "WebAssembly"
    tier = "broad"
    convenience = "none"
    needs_wasm = True
    extension = "h"
    include_guard = "LOUDER_WASM_H"
    header_file = "louder_wasm.h"
    own_banner = [
        "/* Bindings for WebAssembly, generated from the C header of the Louder engine.",
        " *",
        " * These are the signatures the compiled module actually exports, which are not",
        " * the ones a plain C header gives. An aggregate larger than a register is",
        " * returned through a pointer the caller passes as the first argument, and an",
        " * aggregate passed by value is copied into linear memory by the caller and",
        " * arrives as a pointer too. Both differences are already applied below, because",
        " * a caller that gets them wrong reads four bytes that mean something else.",
        " *",
        " * Every size and offset is pinned by a static assertion at the bottom, so a",
        " * compiler that disagrees with the module refuses to build rather than",
        " * producing a program that misreads a struct.",
        " *",
        " * Do not edit. Run `python engine/bindings/generate.py` and this file comes back.",
        " */",
    ]

    # -- files -------------------------------------------------------------

    def build(self, out: Emitted) -> None:
        module = self.options.wasm
        if module is not None and module.is_file():
            out.add_binary(module.name, module.read_bytes())
        out.add_lines(self.header_file, self.render())
        out.add("louder_wasm.json", self.render_json())
        out.add_lines("README.md", self.render_readme())

    def preamble(self) -> list[str]:
        return [
            "#if !defined(__wasm__)",
            '#error "these declarations describe a wasm32 module; compile for wasm32"',
            "#endif",
        ]

    # -- the signatures ----------------------------------------------------

    def exported(self, func: Func):
        """The module's own signature for a function, or None if unavailable."""
        return getattr(func, "wasm", None)

    def signature_types(self, func: Func) -> list[str]:
        """The C types of the arguments the module actually takes."""
        signature = self.exported(func)
        out: list[str] = []
        if signature is not None and signature.sret is not None:
            out.append(self.type_expr(func.ret) + "*")
        for index, param in enumerate(func.params):
            if signature is not None and index in signature.by_value:
                out.append(self.type_expr(param.type) + "*")
                continue
            out.append(self.type_expr(param.type))
        return out

    def signature_return(self, func: Func) -> str:
        signature = self.exported(func)
        if signature is not None and signature.sret is not None:
            return "void"
        return self.type_expr(func.ret)

    def emit_function(self, func: Func) -> list[str]:
        """The declaration the module actually exports.

        Where the header and the module differ, the difference is spelled out in
        the comment rather than left for a caller to infer: the parameter that
        receives the result is named `out`, and one that was passed by value is
        marked. A reader can then tell at a glance which of the two rules a given
        call is under.
        """
        signature = self.exported(func)
        types = self.signature_types(func)
        lines: list[str] = []

        notes: list[str] = []
        if signature is None:
            notes.append("generated without the module, so this is the plain C signature")
        else:
            if signature.sret is not None:
                notes.append("`out` receives the result")
            by_value = [self.type_expr(func.params[i].type) for i in sorted(signature.by_value)]
            if by_value:
                notes.append("passed by value, so " + ", ".join(by_value) + " arrives as a pointer")

        if func.doc:
            lines.extend(self.comment(func.doc))
        if notes:
            lines.extend(self.comment("\n\n".join(notes)))

        params: list[str] = []
        cursor = 0
        if signature is not None and signature.sret is not None:
            params.append(types[0] + " out")
            cursor = 1
        for index, param in enumerate(func.params):
            marker = ""
            if signature is not None and index in signature.by_value:
                marker = " /* by value */"
            params.append(types[cursor + index] + " " + self.names.arg(param.name, index) + marker)

        # An empty list means "no information about the arguments" in C, which is
        # not what a function that takes none declares. `(void)` says it exactly.
        arguments = ", ".join(params) if params else "void"
        returns = self.signature_return(func)
        one_line = "extern " + returns + " " + self.names.func(func.name) + "(" + arguments + ");"
        if len(one_line) <= 100:
            lines.append(one_line)
            return lines

        lines.append("extern " + returns + " " + self.names.func(func.name) + "(")
        for position, param in enumerate(params):
            tail = "," if position + 1 < len(params) else ""
            lines.append("    " + param + tail)
        lines.append(");")
        return lines

    def assertions(self) -> list[str]:
        """Pin every size and offset the model computed, as C would.

        The model and the compiler arrive at the same layout by different routes,
        so a disagreement between them is a real disagreement rather than the
        same code being read twice. One that got through here would be a host
        reading the wrong four bytes, which is why it is a build failure rather
        than a warning.
        """
        lines = [
            "/* Layout, checked by the compiler rather than trusted. */",
            "#if defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L",
            "#define LOUDER_ASSERT(name, cond) _Static_assert(cond, name)",
            "#define LOUDER_ASSERT_FIELD(type, field, offset) \\",
            "    _Static_assert(offsetof(type, field) == offset, "
            '#type "." #field " is not at the offset the module expects")',
        ]
        for name in self.aggregate_order():
            spelled = self.names.type(name)
            union = self.abi.unions.get(name)
            if union is not None:
                lines.append(
                    "LOUDER_ASSERT(\"" + name + " is not the size the module expects\", sizeof("
                    + spelled + ") == " + str(self.layout.size(union)) + ");"
                )
                lines.append(
                    "LOUDER_ASSERT_FIELD(" + spelled + ", body, "
                    + str(self.layout.variant_offset(union, union.variants[0])) + ");"
                )
                continue
            struct = self.abi.structs.get(name)
            if struct is None:
                continue
            lines.append(
                "LOUDER_ASSERT(\"" + name + " is not the size the module expects\", sizeof("
                + spelled + ") == " + str(self.layout.size(struct)) + ");"
            )
            if self.abi.is_union_body(struct):
                continue
            for placed in self.layout.fields(struct):
                if placed.offset == 0:
                    continue
                lines.append(
                    "LOUDER_ASSERT_FIELD(" + spelled + ", " + self.names.field(placed.name)
                    + ", " + str(placed.offset) + ");"
                )
        lines.append("#endif")
        return lines

    # -- the manifest ------------------------------------------------------

    def wasm_of(self, type: Type) -> Optional[str]:
        resolved = self.abi.resolve(type)
        if isinstance(resolved, Pointer):
            return "i32"
        if isinstance(resolved, Scalar):
            try:
                return _WASM_OF_SCALAR[resolved.name]
            except KeyError:
                raise NotImplementedError("no wasm type for " + resolved.name) from None
        return "i32"

    def render_json(self) -> str:
        exports = []
        for func in self.abi.funcs:
            signature = self.exported(func)
            types = self.signature_types(func)
            base = 1 if signature is not None and signature.sret is not None else 0
            entry = {
                "name": func.name,
                "returns": self.signature_return(func),
                "returnsWasm": None if base else self.wasm_of(func.ret),
                "doc": " ".join(func.doc.split()) or None,
                "params": [
                    {
                        "name": param.name,
                        "type": self.type_expr(param.type),
                        "wasm": types[base + index],
                        "byValue": bool(signature is not None and index in signature.by_value),
                    }
                    for index, param in enumerate(func.params)
                ],
            }
            if signature is not None:
                entry["module"] = {"params": list(signature.params), "results": list(signature.results)}
                if signature.sret is not None:
                    entry["returnsThrough"] = 0
            else:
                entry["note"] = "generated without the module, so this is the plain C signature"
            exports.append(entry)

        types = []
        for name in self.aggregate_order():
            spelled = self.names.type(name)
            union = self.abi.unions.get(name)
            if union is not None:
                types.append(
                    {
                        "name": name,
                        "kind": "tagged-union",
                        "size": self.layout.size(union),
                        "align": self.layout.size_align(union)[1],
                        "tag": union.tag_type.name,
                        "bodyOffset": self.layout.variant_offset(union, union.variants[0]),
                        "arms": [
                            {
                                "tag": value,
                                "name": self.enum_member(spelled, member),
                                "carries": self.type_expr(variant.type) if variant is not None else None,
                            }
                            for value, member, variant in self.abi.union_arms(union)
                        ],
                    }
                )
                continue
            struct = self.abi.structs.get(name)
            if struct is None:
                continue
            types.append(
                {
                    "name": name,
                    "kind": "union" if self.abi.is_union_body(struct) else "struct",
                    "size": self.layout.size(struct),
                    "align": self.layout.size_align(struct)[1],
                    "fields": [
                        {
                            "name": placed.name,
                            "type": self.type_expr(placed.type),
                            "offset": placed.offset,
                            "size": placed.size,
                        }
                        for placed in self.layout.fields(struct)
                    ],
                }
            )

        document = {
            "engine": "louder_rust",
            "target": {"triple": "wasm32-unknown-unknown", "pointerSize": 4, "sizeTSize": 4},
            "module": {"exportsMemory": True, "memoryPages": self.memory_pages()},
            "callingConvention": [
                "An aggregate larger than one register is returned through a pointer "
                "the caller passes as the first argument; the function returns nothing.",
                "An aggregate passed by value is copied into linear memory by the "
                "caller and arrives as a pointer.",
                "A pointer and a size_t are both i32 on wasm32.",
            ],
            "aliases": {name: self.type_expr(alias.target) for name, alias in self.abi.aliases.items()},
            "enums": {
                self.names.type(name): {
                    "underlying": self.type_expr(enum.underlying),
                    "variants": {
                        self.enum_member(self.names.type(name), variant): enum.value(variant)
                        for variant in enum.names
                    },
                }
                for name, enum in self.abi.enums.items()
            },
            "constants": {const.name: const.value for const in self.abi.consts},
            "bitflags": {
                name: {
                    "underlying": enum.underlying.name if isinstance(enum.underlying, Scalar) else "int32_t",
                    "flags": dict(enum.variants),
                }
                for name, enum in self.abi.bitflags.items()
            },
            "opaques": sorted(self.abi.opaques),
            "types": types,
            "functions": exports,
        }
        return json.dumps(document, indent=2) + "\n"

    def memory_pages(self) -> Optional[int]:
        module = self.options.wasm
        if module is None or not module.is_file():
            return None
        from ..wasm import load

        return load(module).memory_pages

    # -- the note ----------------------------------------------------------

    def render_readme(self) -> list[str]:
        return [
            "# Louder, compiled to WebAssembly",
            "",
            "Generated. Do not edit; run `python engine/bindings/generate.py` instead.",
            "",
            "## What is here",
            "",
            "| File | What it is |",
            "| --- | --- |",
            "| `louder_wasm.h` | The signatures the module exports, as a C header, with every size pinned by a static assertion. |",
            "| `louder_wasm.json` | The same thing for a host that is not C. |",
            "| `louder_rust.wasm` | The engine, built for `wasm32-unknown-unknown`. |",
            "",
            "## Loading it",
            "",
            "There is no glue code. The module is instantiated and its exports are",
            "called directly.",
            "",
            "```javascript",
            "const bytes = await readFile('louder_rust.wasm');",
            "const { instance } = await WebAssembly.instantiate(bytes, {});",
            "const { memory, init_louder, declare_name } = instance.exports;",
            "```",
            "",
            "The module exports its memory as `memory`. It imports nothing, because",
            "the engine has nothing to ask the host for.",
            "",
            "## The two differences from a plain C header",
            "",
            "Everything else about the boundary is the same as the C one. These two",
            "are not, and getting them wrong reads four bytes that mean something",
            "else entirely:",
            "",
            "1. **An aggregate returned by value does not come back in a register.**",
            "   For those functions the caller passes a pointer to its own space as",
            "   the *first* argument and the function returns nothing. In the header",
            "   that argument is called `out`; in the JSON it is `returnsThrough`.",
            "2. **An aggregate passed by value arrives as a pointer.** The caller",
            "   copies the struct into linear memory and passes where it went. In the",
            "   header the parameter is marked `by value`; in the JSON it is",
            "   `byValue: true`.",
            "",
            "A struct small enough to fit in one register is exempt from the first",
            "rule, which is why `get_actor_flags` returns a plain `i32` while",
            "`get_actor_additional` returns nothing.",
            "",
            "## One name difference from the engine's own header",
            "",
            "C keeps struct tags, typedef names, and enum constants in one namespace,",
            "and the engine has a variant of `EventData_Tag` called `ChoiceAppearance`",
            "alongside a struct of the same name. That does not compile, so a constant",
            "that would redeclare something else in the file is spelled with the enum's",
            "name in front of it: `EventData_Tag_ChoiceAppearance`. Nothing else is",
            "renamed.",
            "",
            "## Where the strings go",
            "",
            "Strings cross the boundary as bytes: a `uint8_t*` and a `size_t`. The",
            "engine reads JSON as UTF-8 and stores text as interned names, so a name",
            "crosses as an `int32_t` handle rather than a pointer. Handles come from a",
            "hash, so they are not sequential and cannot be guessed.",
            "",
            "## Reading a name back",
            "",
            "The engine reports how much text fitted rather than refusing, so the",
            "call reads into a buffer and asks again if it filled:",
            "",
            "```javascript",
            "function readName(nameId) {",
            "  for (let size = 4096; ; size *= 4) {",
            "    const at = arena.alloc(size);",
            "    const got = exports.read_name(ctx, nameId, at, size - 1);",
            "    if (got === 0) return '';",
            "    if (got < size - 1) return decoder.decode(memory.buffer, at, got);",
            "  }",
            "}",
            "```",
            "",
            "The JavaScript binding in `out/javascript` has that written out, along",
            "with an arena over the module's own linear memory.",
            "",
        ]