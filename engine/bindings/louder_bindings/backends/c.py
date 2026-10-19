"""C, which is the boundary described in the language it was written in.

There is less to do here than anywhere else: the header is most of the binding.
Two things are still worth generating rather than copying.

The first is that the engine's own header does not compile. `EventData_Tag` has
a variant called `ChoiceAppearance` and there is a struct of the same name, and
C keeps those in one namespace. The constants that would redeclare something are
given the enum's name in front, which is what `ClikeBackend` handles.

The second is the convenience layer in `louder.c`. Every cursor in the header
has the same shape — a `start`, a result, and an end — and every read is a
buffer with a length that the engine reports rather than refuses. Both are
mechanical enough to write once from the model, which is what turns sixteen
near-identical loops into one.
"""

from __future__ import annotations

from ..ir import Func, Named, Pointer, Scalar, Type
from .base import Emitted
from .clike import ClikeBackend, literal

# The buffer size the first read asks for. The engine reports what fitted, so
# this only decides how often the loop runs.
_FIRST_READ = 4096


def _first_line(doc: str) -> str:
    """The opening sentence of an entry point's documentation.

    A C comment in front of a helper is the first thing a reader sees, and the
    engine writes a one-line summary first followed by the detail, so the summary
    is the one worth carrying.
    """
    return " ".join((doc.splitlines() or [""])[0].split())


class CBackend(ClikeBackend):
    name = "c"
    label = "C"
    tier = "broad"
    convenience = "thin"
    needs_library = True
    extension = "h"
    include_guard = "LOUDER_H"
    header_file = "louder.h"
    # `stdlib.h` and `string.h` are here for the convenience layer, which calls
    # `realloc` and `strlen` and so cannot be a header that promises otherwise.
    system_includes = ("<stdbool.h>", "<stddef.h>", "<stdint.h>", "<stdlib.h>", "<string.h>")
    own_banner = [
        "/* Bindings for C, generated from the C header of the Louder engine.",
        " *",
        " * One difference from the header the engine ships: this one compiles. C keeps",
        " * struct tags, typedef names, and enum constants in a single namespace, and the",
        " * engine has a variant of `EventData_Tag` named `ChoiceAppearance` alongside a",
        " * struct of the same name. A constant that would redeclare something else in the",
        " * file is spelled with the enum's name in front of it; nothing else is renamed.",
        " *",
        " * `louder.c` holds the convenience layer: a growable buffer, the string",
        " * arguments as `const char *`, and a walk for each of the cursors.",
        " *",
        " * Do not edit. Run `python engine/bindings/generate.py` and this file comes back.",
        " */",
    ]

    def build(self, out: Emitted) -> None:
        out.add_lines(self.header_file, self.render())
        out.add_lines("louder.c", self.render_source())
        out.add("walk.c", self.runtime("c/walk.c").read_text(encoding="utf-8"))
        out.add_lines("README.md", self.render_readme())

    def epilogue(self) -> list[str]:
        """Everything a host can use without linking anything.

        The convenience layer is `static inline` in the header rather than in a
        separate source file, so there is nothing to add to a build and nothing to
        forget at link time. A C user should not have to know a second file exists.
        """
        lines: list[str] = ["", "/*" + "-" * 74, " * The convenience layer.", " *"]
        lines.append(" * None of this is required. The entry points above are the whole")
        lines.append(" * boundary; everything here is what the sixteen identical loops in a host")
        lines.append(" * would otherwise each be written out as.")
        lines.append(" */")
        lines.extend(self.render_buffer())
        lines.append("")
        lines.extend(self.render_name_helpers())
        lines.extend(self.render_read_helpers())
        lines.extend(self.render_load_helpers())
        for func in self.cursors():
            lines.extend(self.render_cursor(func))
        return lines

    def render_source(self) -> list[str]:
        """A translation unit that includes the header and nothing else.

        The convenience layer is `static inline` and lives in the header, so there
        is nothing here to compile for its sake. What there is to compile for is
        the header itself: this file is the cheapest way for anyone to check
        that the generated code still builds without pulling in their own source.
        """
        return [
            *self.banner_block(),
            "",
            "/*",
            " * Nothing is defined here on purpose. The convenience layer is `static inline`",
            " * in the header, so a host needs only to include it. This translation unit",
            " * exists so that compiling it is a check on the header.",
            " */",
            "",
            '#include "' + self.header_file + '"',
        ]

    # -- the cursors -------------------------------------------------------

    def cursors(self) -> list[Func]:
        """The functions that walk a collection.

        Every one of them takes a cursor named `start` as its last argument and
        returns one element. That is the whole test, and it is a test on the
        model rather than on the names, so a cursor added later is picked up
        without this being told.
        """
        found = []
        for func in self.abi.funcs:
            if not func.params:
                continue
            if func.params[-1].name != "start":
                continue
            if func.name not in ("get_next_speaker_listen_to",) and not func.name.startswith("next_"):
                continue
            found.append(func)
        return found

    def cursor_element(self, func: Func) -> str:
        return self.type_expr(func.ret)

    def cursor_prefix(self, func: Func) -> str:
        """The walk's own name, from the function that drives it.

        `next_hub_by_channel` walks the hubs by channel, so the walk is
        `louder_hubs_by_channel`. `get_next_speaker_listen_to` is the odd one out
        and is spelled by hand here rather than by a rule that would have to
        know it.
        """
        stem = func.name
        for prefix in ("get_next_", "next_"):
            if stem.startswith(prefix):
                stem = stem[len(prefix):]
                break
        return "louder_" + stem

    def sentinel_for(self, func: Func) -> str:
        """The constant the engine returns at the end of this walk.

        The header declares one for `LouderName` and `LouderActor`, and not for
        positions even though the documentation names the one it uses. Rather
        than guess at a name for it, `none_constant` finds the constant when
        there is one and falls back to the value every one of them shares.
        """
        return self.none_constant(func.ret)

    # -- the header --------------------------------------------------------

    def assertions(self) -> list[str]:
        lines = [
            "/* Layout, checked by the compiler rather than trusted. */",
            "#if defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L",
            "#define LOUDER_ASSERT(name, cond) _Static_assert(cond, name)",
            "#define LOUDER_ASSERT_FIELD(type, field, offset) \\",
            "    _Static_assert(offsetof(type, field) == offset, "
            '#type "." #field " is not at the offset the engine expects")',
        ]
        for name in self.aggregate_order():
            spelled = self.names.type(name)
            union = self.abi.unions.get(name)
            if union is not None:
                lines.append(
                    'LOUDER_ASSERT("' + name + ' is not the size the engine expects", sizeof('
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
                'LOUDER_ASSERT("' + name + ' is not the size the engine expects", sizeof('
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

    def render_buffer(self) -> list[str]:
        return [
            "/* A byte buffer that grows. */",
            "/*",
            " * The engine never refuses a buffer: it writes what fits and reports how much",
            " * that was, so the only way to be sure of a whole document is to ask again",
            " * with more room. Two reads is the ordinary case; the loop in each helper",
            " * below is what makes a long one safe.",
            " */",
            "typedef struct {",
            "    uint8_t *data;",
            "    size_t length;",
            "    size_t capacity;",
            "} LouderBuffer;",
            "",
            "static inline void louder_buffer_free(LouderBuffer *buffer) {",
            "    free(buffer->data);",
            "    buffer->data = NULL;",
            "    buffer->length = 0;",
            "    buffer->capacity = 0;",
            "}",
            "",
            "static inline bool louder_buffer_reserve(LouderBuffer *buffer, size_t wanted) {",
            "    size_t capacity = buffer->capacity ? buffer->capacity : " + str(_FIRST_READ) + ";",
            "    uint8_t *grown;",
            "    while (capacity < wanted) {",
            "        capacity *= 2;",
            "    }",
            "    if (capacity <= buffer->capacity) {",
            "        return true;",
            "    }",
            "    grown = (uint8_t *)realloc(buffer->data, capacity);",
            "    if (grown == NULL) {",
            "        return false;",
            "    }",
            "    buffer->data = grown;",
            "    buffer->capacity = capacity;",
            "    return true;",
            "}",
        ]

    def render_name_helpers(self) -> list[str]:
        """Declaring a name from a string, looking first.

        The engine splits this in two on purpose: `get_named_id` answers for a
        string that is already there and `declare_name` refuses to make a second
        id for one that is, returning `LOUDER_NAME_NONE` instead. A host that
        only declared would get a different id every time it asked, so the two
        are put back together here rather than left as a trap.
        """
        declares = self.abi.has_func("declare_name")
        looks_up = self.abi.has_func("get_named_id")
        if not declares:
            return []
        none = self.none_constant(Named("LouderName"))
        lines = [
            "/* The id of some text, declaring it the first time. */",
        ]
        if looks_up:
            lines.extend(
                [
                    "static inline LouderName louder_name(LouderContext *ctx, const char *text) {",
                    "    LouderName found = get_named_id(ctx, (const uint8_t *)text, strlen(text));",
                    "    if (found != " + none + ") {",
                    "        return found;",
                    "    }",
                    "    return declare_name(ctx, (const uint8_t *)text, strlen(text));",
                    "}",
                ]
            )
        else:
            lines.extend(
                [
                    "/* Returns " + none + " if the text was already declared: the engine",
                    " * hands out one id per string, and this is the only way to ask for it. */",
                    "static inline LouderName louder_name(LouderContext *ctx, const char *text) {",
                    "    return declare_name(ctx, (const uint8_t *)text, strlen(text));",
                    "}",
                ]
            )
        lines.append("")
        return lines

    def render_read_helpers(self) -> list[str]:
        """One grow-until-it-fits helper per read that writes into a buffer.

        The shape is the same in every case — the engine writes at most
        `max_size` bytes and says how many it wrote — so the helper is generated
        from the signature rather than written once per function. Whether a
        negative answer is a failure depends only on whether the return type is
        signed, which is a question about the signature and not about the engine.
        """
        signed = ("int8_t", "int16_t", "int32_t", "int64_t", "int", "long long", "ptrdiff_t")
        helpers: list[str] = []
        for func in self.abi.funcs:
            if not func.name.startswith("read_") or len(func.params) < 2:
                continue
            buffer, size = func.params[-2], func.params[-1]
            if buffer.name != "buffer" or size.name != "max_size":
                continue
            if not isinstance(self.abi.resolve(buffer.type), Pointer):
                continue
            resolved = self.abi.resolve(func.ret)
            fails = isinstance(resolved, Scalar) and resolved.name in signed

            leading = func.params[:-2]
            passed = [self.names.arg(param.name, index) for index, param in enumerate(leading)]
            signature = ", ".join(
                self.type_expr(param.type) + " " + self.names.arg(param.name, index)
                for index, param in enumerate(leading)
            )
            call = ", ".join(passed + ["out->data", "size - 1"])
            check = ("    if (written < 0) {", "        return false;", "    }") if fails else ()
            helpers.extend(
                [
                    "/* " + _first_line(func.doc) + " */",
                    "static inline bool louder_" + func.name + "("
                    + (signature + ", " if signature else "") + "LouderBuffer *out) {",
                    "    size_t size = " + str(_FIRST_READ) + ";",
                    "    for (;;) {",
                    "        " + self.type_expr(func.ret) + " written;",
                    "        if (!louder_buffer_reserve(out, size)) {",
                    "            return false;",
                    "        }",
                    "        written = " + self.names.func(func.name) + "(" + call + ");",
                    *check,
                    "        if ((size_t)written + 1 < size) {",
                    "            out->length = (size_t)written;",
                    "            return true;",
                    "        }",
                    "        size *= 2;",
                    "    }",
                    "}",
                    "",
                ]
            )
        return helpers

    def render_load_helpers(self) -> list[str]:
        """The loads that take text, taking a `const char *` instead of a buffer."""
        helpers: list[str] = []
        for func in self.abi.funcs:
            if not func.name.startswith("load_") or len(func.params) < 2:
                continue
            buffer, size = func.params[-2], func.params[-1]
            if buffer.name != "buffer" or size.name != "max_size":
                continue
            if not isinstance(self.abi.resolve(buffer.type), Pointer):
                continue
            leading = func.params[:-2]
            signature = ", ".join(
                self.type_expr(param.type) + " " + self.names.arg(param.name, index)
                for index, param in enumerate(leading)
            )
            passed = [self.names.arg(param.name, index) for index, param in enumerate(leading)]
            arguments = ", ".join(passed + ["(uint8_t *)text", "strlen(text)"])
            helpers.extend(
                [
                    "/* " + _first_line(func.doc) + " */",
                    "static inline " + self.type_expr(func.ret) + " louder_" + func.name + "_text("
                    + (signature + ", " if signature else "") + "const char *text) {",
                    "    return " + self.names.func(func.name) + "(" + arguments + ");",
                    "}",
                    "",
                ]
            )
        return helpers

    def render_cursor(self, func: Func) -> list[str]:
        """One walk type per cursor: the arguments, the cursor, and a step.

        The end of the walk is a value the caller supplies rather than one this
        guesses at. Where the header declares the constant, it is used as the
        default in the note below; where it does not, the note says so rather
        than inventing a name for it.
        """
        leading = func.params[:-1]
        walk = self.cursor_prefix(func)
        struct_name = "Louder" + "".join(w.capitalize() for w in walk[len("louder_"):].split("_"))
        element = self.cursor_element(func)

        fields = [
            (self.type_expr(param.type), self.names.arg(param.name, index))
            for index, param in enumerate(leading)
        ]
        start = self.type_expr(func.params[-1].type)
        sentinel = self.sentinel_for(func)

        lines: list[str] = []
        lines.extend(self.comment(_first_line(func.doc) or ("A walk over " + func.name + ".")))
        lines.append("typedef struct {")
        for spelling, field in fields:
            lines.append("    " + spelling + " " + field + ";")
        lines.append("    " + start + " start;")
        lines.append("    " + start + " end;")
        lines.append("} " + struct_name + ";")
        lines.append("")
        signature = ", ".join(spelling + " " + field for spelling, field in fields)
        lines.append("static inline " + struct_name + " " + walk + "(" + (signature + ", " if signature else "")
                     + start + " start, " + start + " end) {")
        lines.append("    " + struct_name + " walk;")
        for _spelling, field in fields:
            lines.append("    walk." + field + " = " + field + ";")
        lines.append("    walk.start = start;")
        lines.append("    walk.end = end;")
        lines.append("    return walk;")
        lines.append("}")
        lines.append("")
        call = ", ".join(["walk->" + field for _spelling, field in fields] + ["walk->start"])
        lines.append("static inline bool " + walk + "_next(" + struct_name + " *walk, " + element + " *out) {")
        lines.append("    " + element + " found = " + self.names.func(func.name) + "(" + call + ");")
        lines.append("    if (found == walk->end) {")
        lines.append("        return false;")
        lines.append("    }")
        lines.append("    walk->start = found;")
        lines.append("    *out = found;")
        lines.append("    return true;")
        lines.append("}")
        lines.append("")
        if sentinel.startswith("-"):
            lines.append("/* The engine ends this walk with the value its documentation calls")
            lines.append(" * BLANK_POSITION. The header does not declare it, so the walk takes it")
            lines.append(" * as an argument rather than naming one here. */")
        else:
            lines.append("/* The engine ends this walk with " + sentinel + ". */")
        lines.append("")
        return lines

    # -- the note ----------------------------------------------------------

    def render_readme(self) -> list[str]:
        library = self.options.library.name if self.options.library else "louder_rust"
        return [
            "# Louder, from C",
            "",
            "Generated. Do not edit; run `python engine/bindings/generate.py` instead.",
            "",
            "## Files",
            "",
            "| File | What it is |",
            "| --- | --- |",
            "| `louder.h` | The boundary. Every type, constant, and entry point, with the layout pinned by static assertions. |",
            "| `louder.c` | The convenience layer, which a host can ignore entirely. |",
            "| `" + library + "` | The engine itself. |",
            "",
            "## Using it",
            "",
            "```c",
            '#include "louder.h"',
            "",
            "int main(void) {",
            "    LouderContext *engine = init_louder();",
            "    LouderName conversation = louder_name(engine, \"scene\");",
            "",
            "    LouderBuffer json = {0};",
            "    if (!louder_read_state_json(engine, conversation, &json)) {",
            '        louder_buffer_free(&json);',
            "        return 1;",
            "    }",
            "    puts((const char *)json.data);",
            "    louder_buffer_free(&json);",
            "",
            "    deinit_louder(engine);",
            "    return 0;",
            "}",
            "```",
            "",
            "## Compiling",
            "",
            "```",
            "gcc -I. walk.c louder.c -L. -l" + library.replace(".dll", "").replace(".so", "").replace(".dylib", "")
            + " -o walk",
            "```",
            "",
            "## The bits worth knowing",
            "",
            "- **Names are integers.** `LouderName` is an `int32_t` handle into the",
            "  engine intern table. `louder_name` declares text and returns the handle;",
            "  `louder_read_name` reads it back. Handles come from a hash, so they are",
            "  not sequential.",
            "- **Text crosses as bytes.** A `uint8_t *` and a `size_t`, not a",
            "  `const char *`. The engine expects UTF-8 in the JSON and gives back",
            "  exactly the bytes that went in, so decoding is the host's job.",
            "- **Buffers never fail by being small.** A read reports what fitted rather",
            "  than refusing, so `louder_read_name` and friends ask again with more",
            "  room. Write your own loop the same way if you call the header directly.",
            "- **A tagged union is a tag and a body.** `LouderVariant.body.int_` is",
            "  only meaningful when `LouderVariant.tag` says so.",
            "- **Every cursor ends at a value you pass in.** `louder_hubs_by_channel`",
            "  and the rest take both the value to start from and the value that means",
            "  stop, because the header declares that constant for some and not others.",
            "",
            "## One difference from the header the engine ships",
            "",
            "This one compiles. C keeps struct tags, typedef names, and enum constants",
            "in a single namespace, and the engine has a variant of `EventData_Tag`",
            "named `ChoiceAppearance` alongside a struct of the same name. A constant",
            "that would redeclare something else in the file is spelled with the enum's",
            "name in front of it, so it is `EventData_Tag_ChoiceAppearance`. Nothing",
            "else is renamed.",
            "",
        ]