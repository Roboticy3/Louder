"""C++, which is the C boundary plus the things C++ is for.

The C header carries the whole boundary and compiles as C++, so the job here is
not to restate it but to write the parts C++ does better than C: an owning type
for the context, strings instead of buffers, and cursors that read as loops.
That layer is header-only in `louder.hpp`, which is the one file a host adds to
its build.
"""

from __future__ import annotations

from ..ir import Func, Named, Pointer, Scalar
from .base import Emitted
from .c import CBackend


class CppBackend(CBackend):
    name = "cpp"
    label = "C++"
    tier = "broad"
    convenience = "thin"
    needs_library = True
    extension = "hpp"
    include_guard = "LOUDER_H"
    header_file = "louder.h"
    own_banner = [
        "/* Bindings for C++, generated from the C header of the Louder engine.",
        " *",
        " * One difference from the header the engine ships: this one compiles. C keeps",
        " * struct tags, typedef names, and enum constants in a single namespace, and the",
        " * engine has a variant of `EventData_Tag` named `ChoiceAppearance` alongside a",
        " * struct of the same name. A constant that would redeclare something else in the",
        " * file is spelled with the enum's name in front of it; nothing else is renamed.",
        " *",
        " * `louder.hpp` is the layer to use: an owning context, strings, and a loop for",
        " * each cursor. This file is the boundary itself and is what that layer includes.",
        " *",
        " * Do not edit. Run `python engine/bindings/generate.py` and this file comes back.",
        " */",
    ]

    def build(self, out: Emitted) -> None:
        out.add_lines(self.header_file, self.render())
        out.add_lines("louder.hpp", self.render_hpp())
        out.add("walk.cpp", self.runtime("cpp/walk.cpp").read_text(encoding="utf-8"))
        out.add_lines("README.md", self.render_readme())

    def epilogue(self) -> list[str]:
        # The convenience layer is the .hpp, in C++, so the C header itself stays
        # the boundary and nothing else.
        return []

    # -- the layer ---------------------------------------------------------

    def cursor_prefix(self, func: Func) -> str:
        stem = func.name
        for prefix in ("get_next_", "next_"):
            if stem.startswith(prefix):
                stem = stem[len(prefix):]
                break
        return stem

    def render_hpp(self) -> list[str]:
        lines: list[str] = [
            "// Bindings for C++, generated from the C header of the Louder engine.",
            "//",
            "// Do not edit. Run `python engine/bindings/generate.py` and this file comes back.",
            "//",
            "// The C++ layer over `louder.h`. A host adds this one file to its build.",
            "",
            "#ifndef LOUDER_HPP",
            "#define LOUDER_HPP",
            "",
            "#include <cstddef>",
            "#include <cstdint>",
            "#include <stdexcept>",
            "#include <string>",
            "#include <string_view>",
            "#include <utility>",
            "#include <vector>",
            "",
            "extern \"C\" {",
            '#include "louder.h"',
            "}",
            "",
            "namespace louder {",
            "",
        ]
        lines.extend(self.hpp_aliases())
        lines.extend(self.hpp_engine())
        lines.extend(self.hpp_cursors())
        lines.append("} // namespace louder")
        lines.append("")
        lines.append("#endif // LOUDER_HPP")
        lines.append("")
        return lines

    def hpp_aliases(self) -> list[str]:
        lines = [
            "// The boundary, in shorter names.",
            "using Name = LouderName;",
            "using Position = LouderPosition;",
            "using Actor = LouderActor;",
            "using Callback = LouderCallback;",
            "using Expression = LouderExpression;",
            "using LoadErrorCode = LoadError;",
            "",
            "// Why a document was refused.",
            "enum class Failure : LoadErrorCode {",
        ]
        for index, (_value, name) in enumerate(self.code_family()):
            lines.append("    " + name[len("LOUDER_LOAD_"):] + " = " + name + ",")
        lines.extend(
            [
                "};",
                "",
                "/// A document load that was not accepted, with the reason the engine gave.",
                "class LoadFailure : public std::runtime_error {",
                "public:",
                "    explicit LoadFailure(Failure reason)",
                '        : std::runtime_error("the engine refused the document"), reason_(reason) {}',
                "    [[nodiscard]] Failure reason() const noexcept { return reason_; }",
                "",
                "private:",
                "    Failure reason_;",
                "};",
                "",
            ]
        )
        return lines

    def code_family(self) -> list[tuple[int, str]]:
        """The `LOUDER_LOAD_*` constants, as `(value, name)`, in value order."""
        found = [
            (int(c.value), c.name)
            for c in self.abi.consts
            if c.name.startswith("LOUDER_LOAD_") and isinstance(c.value, int)
        ]
        found.sort()
        return found

    def hpp_engine(self) -> list[str]:
        return [
            "/// One engine context.",
            "///",
            "/// Owns the name table, every conversation, and the event queue. The",
            "/// destructor closes it, which is the whole point of the class existing.",
            "class Louder {",
            "public:",
            "    Louder() : ctx_(init_louder()) {",
            "        if (ctx_ == nullptr) {",
            '            throw std::runtime_error("the engine did not start");',
            "        }",
            "    }",
            "    ~Louder() { close(); }",
            "",
            "    Louder(const Louder &) = delete;",
            "    Louder &operator=(const Louder &) = delete;",
            "",
            "    Louder(Louder &&moved) noexcept : ctx_(moved.ctx_) { moved.ctx_ = nullptr; }",
            "    Louder &operator=(Louder &&moved) noexcept {",
            "        if (this != &moved) {",
            "            close();",
            "            ctx_ = moved.ctx_;",
            "            moved.ctx_ = nullptr;",
            "        }",
            "        return *this;",
            "    }",
            "",
            "    /// Release the context and everything it holds.",
            "    void close() noexcept {",
            "        if (ctx_ != nullptr) {",
            "            deinit_louder(ctx_);",
            "            ctx_ = nullptr;",
            "        }",
            "    }",
            "",
            "    /// The raw context pointer, for anything this layer does not wrap.",
            "    [[nodiscard]] LouderContext *handle() const noexcept { return ctx_; }",
            "",
            "    // --- names ---",
            "",
            "    /// The id of some text, or LOUDER_NAME_NONE if it was never declared.",
            "    [[nodiscard]] Name lookup(std::string_view text) const {",
            "        return get_named_id(ctx_,",
            "                            reinterpret_cast<const uint8_t *>(text.data()),",
            "                            text.size());",
            "    }",
            "",
            "    /// The id of some text, declaring it the first time.",
            "    ///",
            "    /// `declare_name` refuses to make a second id for text that already has",
            "    /// one, returning LOUDER_NAME_NONE, so this looks first and declares",
            "    /// second. The engine splits them on purpose; a host should not have to",
            "    /// know.",
            "    [[nodiscard]] Name name(std::string_view text) const {",
            "        const Name found = lookup(text);",
            "        if (found != LOUDER_NAME_NONE) {",
            "            return found;",
            "        }",
            "        return declare_name(ctx_,",
            "                            reinterpret_cast<const uint8_t *>(text.data()),",
            "                            text.size());",
            "    }",
            "",
            "    /// The text behind an id.",
            "    ///",
            "    /// The engine reports how many bytes fitted rather than refusing, so the",
            "    /// only way to be sure of the whole answer is to ask again with more",
            "    /// room. Two calls is the ordinary case.",
            "    [[nodiscard]] std::string text(Name id) const {",
            "        if (id == LOUDER_NAME_NONE) {",
            "            return {};",
            "        }",
            "        for (std::size_t size = 4096;; size *= 4) {",
            "            std::string buffer(size, '\\0');",
            "            const std::size_t got = read_name(ctx_, id,",
            "                                              reinterpret_cast<uint8_t *>(buffer.data()),",
            "                                              size - 1);",
            "            if (got == 0) {",
            "                return {};",
            "            }",
            "            if (got + 1 < size) {",
            "                buffer.resize(got);",
            "                return buffer;",
            "            }",
            "        }",
            "    }",
            "",
            "    // --- documents ---",
            "",
            "    /// Load a conversation from a JSON document.",
            "    void load_conversation(Name conversation, std::string_view document) {",
            "        const LoadErrorCode code = load_conversation_json(",
            "            ctx_, conversation,",
            "            reinterpret_cast<uint8_t *>(const_cast<char *>(document.data())),",
            "            document.size());",
            "        raise(code);",
            "    }",
            "",
            "    /// Load the speaker config, which is a singleton.",
            "    void load_config(std::string_view document) {",
            "        const LoadErrorCode code = load_conversation_config(",
            "            ctx_,",
            "            reinterpret_cast<uint8_t *>(const_cast<char *>(document.data())),",
            "            document.size());",
            "        raise(code);",
            "    }",
            "",
            "    /// Throw when a load was not accepted, with the engine's reason.",
            "    static void raise(LoadErrorCode code) {",
            "        if (code != LOUDER_LOAD_OK) {",
            "            throw LoadFailure(static_cast<Failure>(code));",
            "        }",
            "    }",
            "",
            "    // --- the JSON reads ---",
            "",
            "    /// The whole story state as a JSON document.",
            "    [[nodiscard]] std::string state_json(Name conversation) const {",
            "        return read_growing(",
            "            [&](uint8_t *at, std::size_t size) {",
            "                return read_state_json(ctx_, conversation, at, size);",
            "            });",
            "    }",
            "",
            "    /// The story state and every actor, as a JSON document.",
            "    [[nodiscard]] std::string snapshot_json(Name conversation) const {",
            "        return read_growing(",
            "            [&](uint8_t *at, std::size_t size) {",
            "                return read_snapshot_json(ctx_, conversation, at, size);",
            "            });",
            "    }",
            "",
            "    // --- story state ---",
            "",
            "    /// A story state value, as whatever it carries.",
            "    [[nodiscard]] int get_int(Name conversation, std::string_view key) const {",
            "        const LouderVariant value = get_story_state(ctx_, conversation, name(key));",
            "        return value.tag == Int ? value.body.int_ : 0;",
            "    }",
            "",
            "    [[nodiscard]] float get_float(Name conversation, std::string_view key) const {",
            "        const LouderVariant value = get_story_state(ctx_, conversation, name(key));",
            "        return value.tag == Float ? value.body.float_ : 0.0f;",
            "    }",
            "",
            "    [[nodiscard]] std::string get_text(Name conversation, std::string_view key) const {",
            "        const LouderVariant value = get_story_state(ctx_, conversation, name(key));",
            "        return value.tag == String ? text(value.body.string) : std::string{};",
            "    }",
            "",
            "    /// Write an integer story state value.",
            "    void set_int(Name conversation, std::string_view key, int value) {",
            "        set_story_state(ctx_, conversation, name(key), make_variant_int(value));",
            "    }",
            "",
            "    /// Write a float story state value.",
            "    void set_float(Name conversation, std::string_view key, float value) {",
            "        set_story_state(ctx_, conversation, name(key), make_variant_float(value));",
            "    }",
            "",
            "    /// Write a text story state value, stored as the interned name.",
            "    void set_text(Name conversation, std::string_view key, std::string_view value) {",
            "        set_story_state(ctx_, conversation, name(key),",
            "                        make_variant_string(name(value)));",
            "    }",
            "",
            "private:",
            "    /// Read into a buffer that grows until the whole answer fits.",
            "    template <typename F>",
            "    static std::string read_growing(F &&call) {",
            "        for (std::size_t size = 4096;; size *= 4) {",
            "            std::string buffer(size, '\\0');",
            "            const auto written = call(reinterpret_cast<uint8_t *>(buffer.data()), size - 1);",
            "            if (written < 0) {",
            '                throw std::runtime_error("the engine refused the read");',
            "            }",
            "            if (static_cast<std::size_t>(written) + 1 < size) {",
            "                buffer.resize(static_cast<std::size_t>(written));",
            "                return buffer;",
            "            }",
            "        }",
            "    }",
            "",
            "    LouderContext *ctx_;",
            "};",
            "",
        ]

    def hpp_cursors(self) -> list[str]:
        """One range per cursor, so a walk reads as a range-for.

        A cursor here is any entry point whose last parameter is named `start`:
        the engine hands one element at a time, the caller passes the previous
        one back, and the walk is over when the end value comes back. Sixteen of
        them, all the same shape, so one generated class each rather than sixteen
        copies of a loop.
        """
        lines: list[str] = []
        for func in self.cursors():
            lines.extend(self.hpp_cursor(func))
        return lines

    def hpp_cursor(self, func: Func) -> list[str]:
        stem = self.cursor_prefix(func)
        class_name = "".join(w.capitalize() for w in stem.split("_"))
        leading = func.params[:-1]
        element = self.type_expr(func.ret)
        start_type = self.type_expr(func.params[-1].type)
        sentinel = self.none_constant(func.ret)

        args = ", ".join(self.type_expr(p.type) + " " + self.names.arg(p.name, i) for i, p in enumerate(leading))
        fields = [self.names.arg(p.name, i) for i, p in enumerate(leading)]
        members = [field + "_" for field in fields]
        step = ", ".join(["owner_." + member for member in members] + ["current_"])
        first = ", ".join(members + ["start_"])

        lines = [
            "/// " + _first_line(func.doc),
            "///",
            "/// Reads as `for (const auto hub : louder::" + class_name + "(...))` rather than",
            "/// as the loop it wraps.",
            "class " + class_name + " {",
            "public:",
            "    " + class_name + "(" + (args + ", " if args else "") + start_type + " start, " + start_type + " end)",
        ]
        if members:
            lines.append("        : " + ", ".join(member + "(" + member[:-1] + ")" for member in members) + ",")
            lines.append("          start_(start), end_(end) {}")
        else:
            lines.append("        : start_(start), end_(end) {}")
        lines.extend(
            [
                "",
                "    class Iterator {",
                "    public:",
                "        Iterator(const " + class_name + " &owner, " + element + " current)",
                "            : owner_(owner), current_(current) {}",
                "",
                "        bool operator==(const Iterator &other) const { return current_ == other.current_; }",
                "        " + element + " operator*() const { return current_; }",
                "        Iterator &operator++() {",
                "            current_ = " + self.names.func(func.name) + "(" + step + ");",
                "            return *this;",
                "        }",
                "",
                "    private:",
                "        const " + class_name + " &owner_;",
                "        " + element + " current_;",
                "    };",
                "",
                "    [[nodiscard]] Iterator begin() const {",
                "        return Iterator(*this, " + self.names.func(func.name) + "(" + first + "));",
                "    }",
                "    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }",
                "",
            ]
        )
        if not sentinel.startswith("-"):
            lines.append("    /// The engine ends this walk with " + sentinel + ".")
        else:
            lines.append("    /// The engine ends this walk with the value its documentation calls")
            lines.append("    /// BLANK_POSITION, which the header does not declare.")
        lines.append("private:")
        for i, p in enumerate(leading):
            lines.append("    " + self.type_expr(p.type) + " " + members[i] + ";")
        lines.append("    " + start_type + " start_;")
        lines.append("    " + start_type + " end_;")
        lines.append("};")
        lines.append("")
        return lines

    def render_readme(self) -> list[str]:
        return [
            "# Louder, from C++",
            "",
            "Generated. Do not edit; run `python engine/bindings/generate.py` instead.",
            "",
            "## Files",
            "",
            "| File | What it is |",
            "| --- | --- |",
            "| `louder.h` | The boundary: every type, constant, and entry point, with the layout pinned by static assertions. |",
            "| `louder.hpp` | The layer to use: an owning context, strings, and a loop for each cursor. |",
            "| `walk.cpp` | A complete example, run against the real library. |",
            "",
            "## Using it",
            "",
            "```cpp",
            '#include "louder.hpp"',
            "",
            "int main() {",
            "    louder::Louder engine;",
            "    const louder::Name conversation = engine.name(\"scene\");",
            "",
            "    engine.set_int(conversation, \"mood\", 7);",
            '    std::puts(engine.state_json(conversation).c_str());',
            "}",
            "```",
            "",
            "## The bits worth knowing",
            "",
            "- **One file, header-only.** `louder.hpp` includes `louder.h` inside",
            "  `extern \"C\"`, and there is nothing to link but the engine itself.",
            "- **The context closes itself.** `louder::Louder` is move-only and RAII,",
            "  so there is no `deinit` to forget.",
            "- **Names are integers.** `louder::Name` is an `int32_t` handle. Handles",
            "  come from a hash, so they are not sequential and cannot be guessed.",
            "- **Every cursor is a loop.** `for (const auto hub :",
            "  louder::HubByChannel(...))` is the walk the header would have you write",
            "  out by hand.",
            "",
        ]


def _first_line(doc: str) -> str:
    """The opening sentence of an entry point's documentation."""
    return " ".join((doc.splitlines() or [""])[0].split())