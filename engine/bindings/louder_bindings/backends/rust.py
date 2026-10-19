"""Rust bindings.

Two modules. `ffi` is the boundary as the header writes it: `#[repr(C)]` types,
tagged unions as a tag plus a `union`, and the `extern "C"` block the engine
exports. `lib` is the same engine in Rust terms, and it is what a caller should
use: it takes `&str` and hands back `String`, it owns the buffers the engine
fills, and it turns the `next_*` cursors into iterators.

Rust is the one language here that can express the tagged unions exactly as
they are, so this backend is the least lossy of them all, which makes it the
one to read when checking the others against the header.
"""

from __future__ import annotations

from ..ir import Enum, Named, Pointer, Scalar, Struct, TaggedUnion, Type
from ..naming import Names, screaming, snake
from .base import Backend, Emitted

_SCALARS = {
    "bool": "bool",
    "int8_t": "i8",
    "uint8_t": "u8",
    "int16_t": "i16",
    "uint16_t": "u16",
    "int32_t": "i32",
    "uint32_t": "u32",
    "int": "i32",
    "unsigned": "u32",
    "int64_t": "i64",
    "uint64_t": "u64",
    "float": "f32",
    "double": "f64",
    "size_t": "usize",
    "ptrdiff_t": "isize",
}

# What an enum's underlying type is in Rust, which cbindgen hides behind a guard
# so the header stays C17.
_ENUM_WIDTH = {
    "uint8_t": "u8",
    "int8_t": "i8",
    "uint16_t": "u16",
    "int16_t": "i16",
    "uint32_t": "u32",
    "int32_t": "i32",
    "uint64_t": "u64",
    "int64_t": "i64",
    "size_t": "usize",
}


class RustBackend(Backend):
    name = "rust"
    label = "Rust"
    tier = "solid"
    convenience = "full"
    extension = "rs"
    needs_library = True

    def make_names(self) -> Names:
        # The header's spelling is kept. `EventData_Tag` says what it is in a way
        # `EventDataTag` does not, and the safe layer below refers to these
        # names; recasing one set and not the other would only make them harder
        # to match up.
        return Names("rust", type_case=str, func_case=snake, field_case=snake, const_case=screaming)

    def comment_prefix(self) -> str:
        # `//`, not `///`: the banner lands at the top of a file, where an outer
        # doc comment is an error and an inner one is what is wanted.
        return "//"

    def build(self, out: Emitted) -> None:
        out.add_lines("louder/src/ffi.rs", self.render_ffi())
        out.add_lines("louder/src/lib.rs", self.render_lib())
        out.add_lines("louder/build.rs", self.render_build_script())
        out.add_lines("louder/Cargo.toml", self.render_cargo())
        out.add_lines("louder/examples/walk.rs", self.render_example())
        out.add_lines("louder/README.md", self.render_readme())

    # ------------------------------------------------------------------
    # Types
    # ------------------------------------------------------------------

    def rust_type(self, type: Type) -> str:
        if isinstance(type, Scalar) and type.name == "void":
            # A C function that returns nothing is one that returns `()` here.
            return "()"
        if isinstance(type, Pointer):
            return "*mut " + self.rust_type(type.to)
        if isinstance(type, Scalar):
            return _SCALARS.get(type.name, self.names.type(type.name))
        if isinstance(type, Named):
            alias = self.abi.aliases.get(type.name)
            if alias is not None:
                return self.rust_type(alias.target)
            return self.names.type(type.name)
        raise KeyError(repr(type))

    def argument_type(self, param) -> str:
        """How a parameter is written.

        A pointer to `const` in C becomes `*const` here, which is the whole
        point of keeping the qualifier in the model: it says the engine will not
        write through the buffer, and a caller handing over a shared one should
        be able to say so.
        """
        if isinstance(param.type, Pointer):
            return ("*const " if param.type.const else "*mut ") + self.rust_type(param.type.to)
        return self.rust_type(param.type)

    def member(self, variant: str) -> str:
        return variant + "_" if variant in self.names.reserved else variant

    def arm_field(self, field: str) -> str:
        """An arm's name as a Rust identifier.

        `cbindgen` appends an underscore where the snake-case name would be a C
        keyword, which is how the arm `Int` becomes the member `int_`. Keeping
        the underscore would put `LouderVariantArm::int_` in a caller's hands,
        which says nothing about the engine, so it goes.
        """
        name = self.names.field(field)
        return name.rstrip("_") if name.endswith("_") else name

    def body_field(self, field: str) -> str:
        """The union member's own name, which does keep the underscore."""
        return self.names.field(field)

    def library_stem(self) -> str:
        """The name the engine's library carries, without its extension."""
        return "louder_rust"

    def doc(self, text: str, indent: str = "") -> list[str]:
        if not text:
            return []
        return [indent + "/// " + " ".join(p.split()) for p in text.split("\n\n")]

    def module_banner(self) -> list[str]:
        """The banner as an inner doc comment.

        At the top of a module an outer doc comment is an error, so the three
        lines are re-spelled rather than taken as they are.
        """
        return ["//! " + line[2:].strip() for line in self.banner()]

    # ------------------------------------------------------------------
    # What cbindgen writes, said again in Rust
    # ------------------------------------------------------------------

    def holds_union(self, owner: str, body: str, seen: frozenset[str] = frozenset()) -> bool:
        if owner in seen:
            return False
        seen = seen | {owner}
        struct = self.abi.structs.get(owner)
        if struct is None:
            return False
        for field in struct.fields:
            target = self.underlying_name(field.type)
            if target is None:
                continue
            if target == body or self.holds_union(target, body, seen):
                return True
        return False

    def derives(self, name: str) -> str:
        """What can be derived about a struct.

        `Debug` would need a `Debug` on a union, which cannot have one, and
        `Default` would need to invent a value for a tag that is not there yet,
        except where both are written out by hand, which `emit_union` does for
        every tagged union. So a struct holding one gets neither derived.
        """
        for union in self.abi.unions.values():
            if self.holds_union(name, union.name + "_Body"):
                return "Clone, Copy"
        return "Clone, Copy, Debug, Default"

    def tag_is_total(self, union: TaggedUnion) -> bool:
        """Whether every arm of the tag enum is one this file lists.

        Always true for a C enum derived from a Rust one, so the catch-all is
        only emitted when it is not. That is what keeps a tag this build does not
        know about from being read as whichever arm shares its bits.
        """
        tag = self.abi.enums.get(union.tag_type.name)
        return tag is not None and len(tag.variants) == len(self.abi.union_arms(union))

    def emit_enum(self, enum: Enum) -> list[str]:
        underlying = enum.underlying.name if isinstance(enum.underlying, Scalar) else "int32_t"
        lines = self.doc(enum.doc)
        lines.append("#[repr(" + _ENUM_WIDTH.get(underlying, "i32") + ")]")
        lines.append("#[derive(Clone, Copy, Debug, PartialEq, Eq)]")
        lines.append("pub enum " + self.names.type(enum.name) + " {")
        value = 0
        for variant, explicit in enum.variants:
            value = explicit if explicit is not None else value
            lines.append("    " + self.member(variant) + " = " + str(value) + ",")
            value += 1
        lines.append("}")
        return lines

    def emit_struct(self, struct: Struct) -> list[str]:
        lines = self.doc(struct.doc)
        lines.append("#[repr(C)]")
        lines.append("#[derive(" + self.derives(struct.name) + ")]")
        lines.append("pub struct " + self.names.type(struct.name) + " {")
        for placed in self.layout.fields(struct):
            lines.append("    pub " + self.names.field(placed.name) + ": " + self.rust_type(placed.type) + ",")
        lines.append("}")
        return lines

    def emit_union_body(self, struct: Struct) -> list[str]:
        return self.doc(struct.doc) + [
            "/// Every arm of a union, sharing one block of memory.",
            "///",
            "/// Only `Copy` is derived. `Debug` cannot be, because a union has no",
            "/// single value to describe, and `Default` has nothing to pick.",
            "#[repr(C)]",
            "#[derive(Clone, Copy)]",
            "pub union " + self.names.type(struct.name) + " {",
        ] + [
            "    pub " + self.names.field(placed.name) + ": " + self.rust_type(placed.type) + ","
            for placed in self.layout.fields(struct)
        ] + ["}"]

    def emit_union(self, union: TaggedUnion) -> list[str]:
        arms = [a for a in self.abi.union_arms(union) if a[2] is not None]
        arm_type = union.name + "Arm"
        tag = self.rust_type(union.tag_type)
        name = self.names.type(union.name)

        lines = self.doc(union.doc)
        lines.extend(
            [
                "/// `Debug` and `Default` are written out below rather than derived:",
                "/// the body is a union, which has neither, and the tag has no sensible",
                "/// zero.",
                "#[repr(C)]",
                "#[derive(Clone, Copy)]",
                "pub struct " + name + " {",
                "    pub tag: " + tag + ",",
                "    pub body: " + self.names.type(union.name + "_Body") + ",",
                "}",
                "",
                "/// The arm a " + union.name + " carries, read out of its shared memory.",
                "#[derive(Clone, Copy, Debug)]",
                "pub enum " + arm_type + " {",
            ]
        )
        for _value, _name, carrier in arms:
            lines.append("    " + self.arm_field(carrier.name) + "(" + self.rust_type(carrier.type) + "),")
        lines.extend(["}", ""])

        lines.extend(
            [
                "impl std::fmt::Debug for " + name + " {",
                "    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {",
                "        match self.arm() {",
                "            None => write!(out, concat!(stringify!(" + name + '), " {{ tag: {:?} }}"), self.tag),',
            ]
        )
        for _value, _name, carrier in arms:
            field = self.arm_field(carrier.name)
            lines.append(
                "            Some(" + arm_type + "::" + field + "(value)) => "
                + 'write!(out, concat!("' + name + "::" + field + '", " {{ {:?} }}"), value),'
            )
        lines.extend(
            [
                "        }",
                "    }",
                "}",
                "",
                "impl Default for " + name + " {",
                "    /// The zeroed union.",
                "    ///",
                "    /// Which arm that is depends on the engine, so this is only useful as",
                "    /// something to write into rather than as a value.",
                "    fn default() -> Self {",
                "        " + name + " {",
                "            tag: unsafe { std::mem::zeroed() },",
                "            body: unsafe { std::mem::zeroed() },",
                "        }",
                "    }",
                "}",
                "",
                "impl " + name + " {",
                "    /// The arm the tag names, or `None` for an arm that carries nothing.",
                "    ///",
                "    /// A tag this build does not know about is `None` too, rather than",
                "    /// being read as whichever arm happens to share its bits.",
                "    pub fn arm(&self) -> Option<" + arm_type + "> {",
                "        match self.tag {",
            ]
        )
        for _value, arm_name, carrier in self.abi.union_arms(union):
            spelled = tag + "::" + self.member(arm_name)
            if carrier is None:
                lines.append("            " + spelled + " => None,")
            else:
                lines.append(
                    "            " + spelled + " => Some(" + arm_type + "::" + self.arm_field(carrier.name)
                    + "(unsafe { self.body." + self.body_field(carrier.name) + " })),"
                )
        if not self.tag_is_total(union):
            lines.append("            _ => None,")
        lines.extend(["        }", "    }"])
        lines.extend(self.kind_method(union))
        lines.append("}")
        return lines

    def kind_method(self, union: TaggedUnion) -> list[str]:
        """A `kind` method, for the one union whose arms are worth naming.

        Most tagged unions are an implementation detail of a value; an event's
        kind is something a caller puts in a log and branches on, so it gets the
        name the engine gave it rather than a tag to compare by hand.
        """
        if union.name != "EventData":
            return []
        tag = self.rust_type(union.tag_type)
        lines = [
            "",
            "    /// What kind of event this is, under the name the engine gave it.",
            "    ///",
            "    /// Written out from the tag enum, so an event kind added in Rust shows",
            "    /// up here without anything having to be remembered to add it.",
            "    pub fn kind(&self) -> Option<&'static str> {",
            "        Some(match self.tag {",
        ]
        for _value, name, _body in self.abi.union_arms(union):
            lines.append("            " + tag + "::" + self.member(name) + ' => "' + name + '",')
        if not self.tag_is_total(union):
            lines.append("            _ => return None,")
        lines.extend(["        })", "    }"])
        return lines

    def emit_opaque(self, opaque) -> list[str]:
        """An opaque handle.

        C spells one as a bare tag, and Rust has no such thing: an empty
        `repr(C)` struct is never looked inside, because the header never says
        what is in it.
        """
        return self.doc(opaque.doc) + [
            "#[repr(C)]",
            "#[derive(Debug)]",
            "pub struct " + self.names.type(opaque.name) + " {",
            "    _opaque: [u8; 0],",
            "}",
        ]

    def const_name(self, name: str) -> str:
        """A constant's name, spelled the way Rust spells constants.

        The header writes `ActorFlags_SPECTATOR`, which is the type name run
        into the value name. Rust wants the whole thing shouted, and so does
        anyone scanning for `ACTOR_FLAGS_` rather than for `Flags`.
        """
        if "_" in name:
            head, _sep, tail = name.rpartition("_")
            return screaming(snake(head) + "_" + tail)
        return screaming(name)

    def consts_block(self) -> list[str]:
        lines = ["", "// --- constants " + "-" * 50, ""]
        for const in self.abi.consts:
            name = self.const_name(const.name)
            if isinstance(const.type, Named):
                lines.append(
                    "pub const " + name + ": " + self.names.type(const.type) + " = "
                    + self.names.type(const.type) + " { bits: " + str(int(const.value)) + " };"
                )
            else:
                lines.append("pub const " + name + ": " + self.rust_type(const.type) + " = " + str(int(const.value)) + ";")
        return lines

    def extern_block(self) -> list[str]:
        """Every entry point, declared the way C declares one.

        A plain `extern "C"` block against the library by name, which is the
        ordinary way to bind to a C library in Rust and lets the linker check
        that every symbol named here exists.
        """
        lines = [
            "",
            "// --- entry points " + "-" * 51,
            "",
            "// The engine is a plain C ABI, so the whole surface is one link against",
            "// one library. `build.rs` points the linker at wherever cargo left it.",
            "#[link(name = \"" + self.library_stem() + "\")]",
            "unsafe extern \"C\" {",
        ]
        for func in self.ordered_functions():
            lines.extend(self.doc(func.doc, "    "))
            args = ", ".join(
                self.names.arg(p.name, i) + ": " + self.argument_type(p) for i, p in enumerate(func.params)
            )
            lines.append("    pub fn " + self.names.func(func.name) + "(" + args + ") -> " + self.rust_type(func.ret) + ";")
        lines.append("}")
        return lines

    def render_ffi(self) -> list[str]:
        lines = self.module_banner()
        lines.extend(
            [
                "//! The C boundary, exactly as the header describes it.",
                "//!",
                "//! Nothing here interprets anything. A name is an integer, a tagged union",
                "//! is a tag and a union, and a string is a length and a buffer. The safe",
                "//! layer in the crate root is where those become names, variants and text.",
                "",
                "#![allow(non_camel_case_types, non_snake_case, dead_code)]",
                "",
            ]
        )
        for enum in self.ordered_enums():
            lines.extend(self.emit_enum(enum))
            lines.append("")
        for name in self.declaration_order():
            union = self.abi.unions.get(name)
            if union is not None:
                lines.extend(self.emit_union(union))
            elif name in self.abi.structs:
                struct = self.abi.structs[name]
                lines.extend(self.emit_union_body(struct) if name in self.abi.union_bodies else self.emit_struct(struct))
            else:
                continue
            lines.append("")
        for opaque in self.ordered_opaques():
            lines.extend(self.emit_opaque(opaque))
            lines.append("")
        for alias in self.ordered_aliases():
            lines.extend(self.doc(alias.doc))
            lines.append("pub type " + self.names.type(alias.name) + " = " + self.rust_type(alias.target) + ";")
            lines.append("")
        lines.extend(self.consts_block())
        lines.extend(self.extern_block())
        return lines

    # ------------------------------------------------------------------
    # The safe layer
    # ------------------------------------------------------------------

    def code_family(self) -> list[tuple[int, str]]:
        """The constants that are the codes of the integer alias named `LoadError`.

        `cbindgen` writes a Rust `pub type LoadError = i32` as a typedef and its
        codes as constants beside it, with nothing joining the two up. The join
        available here is the naming: a constant is one of the codes when the
        word after the prefix shared by all of them is a word the alias's own
        name also spells out.
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
        out = [
            (int(c.value), "".join(w[:1].upper() + w[1:].lower() for w in c.name[len(shared) + len(prefix) :].split("_") if w))
            for c, _rest in family
        ]
        return sorted(out)

    def render_lib(self) -> list[str]:
        lines = self.module_banner()
        lines.extend(
            [
                "//! The Louder engine, from Rust.",
                "//!",
                "//! A [`Louder`] owns one engine context and everything that comes with it:",
                "//! the name table, every conversation, and the event queue. Closing it",
                "//! releases all of that at once.",
                "//!",
                "//! Strings cross the boundary as bytes and are decoded here as UTF-8, which",
                "//! is the encoding the engine reads JSON in. Anything else is handed over",
                "//! untouched.",
                "",
                "use std::ffi::CString;",
                "use std::ptr;",
                "",
                "pub mod ffi;",
                "",
                "use ffi::*;",
                "",
            ]
        )
        lines.extend(self.failure_enum())
        lines.extend(self.event_types())
        lines.extend(self.RUNTIME.strip("\n").splitlines())
        return lines

    def failure_enum(self) -> list[str]:
        """Why a document was refused, as the codes the header declares."""
        lines = [
            "/// Why a document was refused.",
            "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
            "pub enum Failure {",
        ]
        for index, (_value, name) in enumerate(self.code_family()):
            if index == 0:
                lines.append("    /// Nothing was refused.")
            else:
                lines.append("    /// One of the ways the engine refuses a document.")
                lines.append("    ///")
                lines.append("    /// The constant of the same name in `ffi` says which.")
            lines.append("    " + name + ",")
        lines.extend(
            [
                "    /// A code this build does not know about.",
                "    ///",
                "    /// What happens when a host carries a document written for a newer engine.",
                "    Other(i32),",
                "}",
                "",
                "impl std::fmt::Display for Failure {",
                "    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {",
                '        write!(out, "{:?}", self)',
                "    }",
                "}",
                "",
                "impl std::error::Error for Failure {}",
                "",
            ]
        )
        return lines

    def event_types(self) -> list[str]:
        """What an event is, once decoded.

        The tag is a name and the arm is a value, because that is what a caller
        does with an event: put the kind in a log, and read the value when the
        kind is one it acts on.
        """
        return [
            "/// One thing that happened, with the names in it already decoded.",
            "#[derive(Debug, Clone)]",
            "pub struct Event {",
            "    /// The conversation it happened in.",
            "    pub conversation: String,",
            "    /// The channel it happened on.",
            "    pub channel: String,",
            "    /// The actor it happened to, or `ACTOR_NONE`.",
            "    pub actor: i32,",
            "    /// Which of the engine's event kinds this is.",
            "    pub kind: &'static str,",
            "    /// Whatever that kind carries, or `None` where it carries nothing.",
            "    pub value: Option<EventValue>,",
            "}",
            "",
            "/// The arms of an event, as values rather than as memory.",
            "#[derive(Debug, Clone)]",
            "pub enum EventValue {",
            "    /// One move of an actor from one hub to another.",
            "    Motion {",
            "        source_position: i32,",
            "        target_position: i32,",
            "        choice_index: i32,",
            "        background: bool,",
            "    },",
            "    /// A choice becoming available on a hub.",
            "    ChoiceAppearance { position: i32, choice_index: i32 },",
            "    /// An arm that carries a name, already decoded.",
            "    Name(String),",
            "    /// An arm this build does not know how to read.",
            "    Other,",
            "}",
            "",
        ]

    RUNTIME = '''
const NAME_BUFFER: usize = 4096;
const JSON_BUFFER: usize = 1 << 16;

/// A buffer the engine fills, growing until the whole answer fits.
///
/// The engine truncates rather than refusing and reports how much fitted, so
/// the only way to be sure of the whole answer is to ask again with more room.
/// Two calls is the normal case; the loop is what makes a long document safe.
fn read_into(call: impl Fn(*mut u8, usize) -> isize, mut size: usize) -> Result<Vec<u8>, Failure> {
    loop {
        let mut buffer = vec![0u8; size];
        let written = call(buffer.as_mut_ptr(), size - 1);
        if written < 0 {
            return Err(Failure::Other(written as i32));
        }
        if written < (size - 1) as isize {
            buffer.truncate(written as usize);
            return Ok(buffer);
        }
        size *= 4;
    }
}

/// Text and its length, which is how every string crosses the boundary.
///
/// `CString` is used because the engine wants a NUL-terminated buffer even where
/// it is also told the length, and because a `String` may contain an interior
/// NUL that would otherwise truncate silently.
fn with_bytes(text: &str) -> Result<(CString, usize), Failure> {
    let raw = CString::new(text).map_err(|_| Failure::Other(-1))?;
    let length = raw.as_bytes().len();
    Ok((raw, length))
}

/// A borrowed buffer as the engine wants to see it.
///
/// The engine only reads through the ones handed to `get_named_id` and
/// `declare_name`, which is what the `const` in the header says. Casting here
/// rather than at each call site keeps that in one place, and the `CString` owns
/// the bytes for as long as the call needs them.
fn as_read_only(raw: &CString) -> *mut u8 {
    raw.as_ptr() as *mut u8
}

/// A list of name ids as the array the engine reads.
///
/// Empty is a one-element array rather than a null pointer, because the engine
/// accepts either and the count is what says how long it is.
fn name_array(ids: &[i32]) -> Vec<i32> {
    if ids.is_empty() { vec![0] } else { ids.to_vec() }
}

/// One engine context.
///
/// Owns the name table, every conversation loaded into it, and the event queue.
/// Closing it, or dropping it, releases all of that at once.
pub struct Louder {
    ctx: *mut LouderContext,
}

impl Louder {
    /// Open a context. The engine is linked at build time; see `build.rs`.
    pub fn new() -> Result<Louder, Failure> {
        let ctx = unsafe { ffi::init_louder() };
        if ctx.is_null() {
            return Err(Failure::Other(-1));
        }
        Ok(Louder { ctx })
    }

    /// Release the context and everything it holds.
    pub fn close(&mut self) {
        if !self.ctx.is_null() {
            unsafe { ffi::deinit_louder(self.ctx) };
            self.ctx = ptr::null_mut();
        }
    }

    /// The raw context pointer, for anything this layer does not wrap.
    pub fn handle(&self) -> *mut LouderContext {
        self.ctx
    }

    // --- names ---

    /// The id of some text, or `None` if it was never declared.
    pub fn lookup(&mut self, text: &str) -> Option<i32> {
        let (raw, length) = with_bytes(text).ok()?;
        let found = unsafe { ffi::get_named_id(self.ctx, as_read_only(&raw), length) };
        if found == LOUDER_NAME_NONE { None } else { Some(found) }
    }

    /// The id of some text, declaring it the first time.
    pub fn declare(&mut self, text: impl AsRef<str>) -> i32 {
        let text = text.as_ref();
        if let Some(found) = self.lookup(text) {
            return found;
        }
        let (raw, length) = match with_bytes(text) {
            Ok(pair) => pair,
            Err(_) => return LOUDER_NAME_NONE,
        };
        unsafe { ffi::declare_name(self.ctx, as_read_only(&raw), length) }
    }

    /// The text behind an id, decoded as UTF-8.
    pub fn text(&mut self, name: i32) -> String {
        if name == LOUDER_NAME_NONE {
            return String::new();
        }
        let mut size = NAME_BUFFER;
        loop {
            let mut buffer = vec![0u8; size];
            let got = unsafe { ffi::read_name(self.ctx, name, buffer.as_mut_ptr(), size - 1) };
            if got == 0 {
                return String::new();
            }
            if got < size - 1 {
                buffer.truncate(got);
                return String::from_utf8_lossy(&buffer).into_owned();
            }
            size *= 4;
        }
    }

    // --- documents ---

    /// Load a conversation from a JSON document.
    pub fn load_conversation(&mut self, conversation: &str, document: &str) -> Result<(), Failure> {
        let name = self.declare(conversation);
        let raw = CString::new(document).map_err(|_| Failure::Other(-1))?;
        let code = unsafe {
            ffi::load_conversation_json(self.ctx, name, raw.as_ptr() as *mut u8, raw.as_bytes().len())
        };
        Self::reason(code).map(|_| ())
    }

    /// Load the speaker config, which is a singleton.
    pub fn load_config(&mut self, document: &str) -> Result<(), Failure> {
        let raw = CString::new(document).map_err(|_| Failure::Other(-1))?;
        let code =
            unsafe { ffi::load_conversation_config(self.ctx, raw.as_ptr() as *mut u8, raw.as_bytes().len()) };
        Self::reason(code).map(|_| ())
    }

    /// Remove every speaker. Conversations are left alone.
    pub fn clear_config(&mut self) -> bool {
        unsafe { ffi::clear_config(self.ctx) == 0 }
    }

    /// Remove a conversation and the actors in it.
    pub fn remove_conversation(&mut self, conversation: &str) -> bool {
        let name = self.declare(conversation);
        unsafe { ffi::remove_conversation(self.ctx, name) == 0 }
    }

    /// The code a load returned, as whatever it means.
    ///
    /// The constants are the ones the header declares rather than the numbers
    /// written out, so a renumbering in the engine is not also something to
    /// chase here.
    fn reason(code: i32) -> Result<(), Failure> {
        match code {
            LOUDER_LOAD_OK => Ok(()),
            LOUDER_LOAD_MALFORMED_JSON => Err(Failure::MalformedJson),
            LOUDER_LOAD_NOT_A_LIST => Err(Failure::NotAList),
            LOUDER_LOAD_NOT_AN_OBJECT => Err(Failure::NotAnObject),
            LOUDER_LOAD_ALREADY_LOADED => Err(Failure::AlreadyLoaded),
            other => Err(Failure::Other(other)),
        }
    }

    // --- actors ---

    /// Put an actor into a conversation and return its id.
    pub fn add_actor(
        &mut self,
        conversation: &str,
        position: i32,
        speak_as: &[&str],
        listen_to: &[&str],
    ) -> i32 {
        let name = self.declare(conversation);
        let speaks: Vec<i32> = speak_as.iter().map(|s| self.declare(s)).collect();
        let listens: Vec<i32> = listen_to.iter().map(|s| self.declare(s)).collect();
        // Both vectors have to outlive the call, which is why they are bound
        // before it rather than being temporaries at the end of an argument.
        let speak_ids = name_array(&speaks);
        let listen_ids = name_array(&listens);
        unsafe {
            ffi::add_actor(
                self.ctx,
                name,
                position,
                ActorFlags { bits: 0 },
                speak_ids.as_ptr(),
                speaks.len() as i32,
                listen_ids.as_ptr(),
                listens.len() as i32,
            )
        }
    }

    /// Take an actor out of a conversation.
    pub fn remove_actor(&mut self, conversation: &str, actor: i32) -> bool {
        let name = self.declare(conversation);
        unsafe { ffi::remove_actor(self.ctx, name, actor) == 0 }
    }

    /// Where an actor is standing.
    pub fn actor_position(&mut self, conversation: &str, actor: i32) -> i32 {
        let name = self.declare(conversation);
        unsafe { ffi::get_actor_position(self.ctx, name, actor) }
    }

    /// Which channel an actor is on.
    pub fn actor_channel(&mut self, conversation: &str, actor: i32) -> String {
        let name = self.declare(conversation);
        self.text(unsafe { ffi::get_actor_channel(self.ctx, name, actor) })
    }

    /// The priority on an actor.
    pub fn actor_priority(&mut self, conversation: &str, actor: i32) -> i32 {
        let name = self.declare(conversation);
        unsafe { ffi::get_actor_priority(self.ctx, name, actor) }
    }

    /// Take a choice on the hub an actor is standing on.
    pub fn choose(&mut self, conversation: &str, actor: i32, index: i32) -> Result<(), Failure> {
        let name = self.declare(conversation);
        if unsafe { ffi::select_choice(self.ctx, name, actor, index) } == 0 {
            Ok(())
        } else {
            Err(Failure::Other(-1))
        }
    }

    /// Move an actor as though it had walked there.
    pub fn warp(&mut self, conversation: &str, actor: i32, target: i32) -> bool {
        let name = self.declare(conversation);
        unsafe { ffi::force_warp(self.ctx, name, actor, target, LOUDER_NAME_NONE, 0) == 0 }
    }

    /// Move an actor straight there.
    pub fn teleport(&mut self, conversation: &str, actor: i32, target: i32) -> bool {
        let name = self.declare(conversation);
        unsafe { ffi::force_position(self.ctx, name, actor, target, LOUDER_NAME_NONE, 0) == 0 }
    }

    // --- hubs ---

    /// The text of a hub.
    pub fn hub_text(&mut self, conversation: &str, position: i32) -> String {
        let name = self.declare(conversation);
        self.text(unsafe { ffi::get_hub_text(self.ctx, name, position) })
    }

    /// The speaker of a hub.
    pub fn hub_speaker(&mut self, conversation: &str, position: i32) -> String {
        let name = self.declare(conversation);
        self.text(unsafe { ffi::get_hub_speaker(self.ctx, name, position) })
    }

    /// How many choices are available to an actor on a hub.
    pub fn hub_choice_count(&mut self, conversation: &str, position: i32, actor: i32) -> i32 {
        let name = self.declare(conversation);
        unsafe { ffi::get_hub_choice_count(self.ctx, name, position, actor) }
    }

    /// The text of one choice, which is the text of the hub it leads to.
    pub fn hub_choice_text(
        &mut self,
        conversation: &str,
        position: i32,
        actor: i32,
        index: i32,
    ) -> String {
        let name = self.declare(conversation);
        self.text(unsafe { ffi::get_hub_choice_text(self.ctx, name, position, actor, index) })
    }

    // --- story state ---

    /// The raw variant behind a story state value.
    pub fn variant(&mut self, conversation: &str, key: &str) -> Option<LouderVariantArm> {
        let name = self.declare(conversation);
        let state = self.declare(key);
        unsafe { ffi::get_story_state(self.ctx, name, state) }.arm()
    }

    /// A story state value as an integer, or zero if there is none.
    pub fn get_int(&mut self, conversation: &str, key: &str) -> i32 {
        match self.variant(conversation, key) {
            Some(LouderVariantArm::int(value)) => value,
            _ => 0,
        }
    }

    /// A story state value as a float, or zero if there is none.
    pub fn get_float(&mut self, conversation: &str, key: &str) -> f32 {
        match self.variant(conversation, key) {
            Some(LouderVariantArm::float(value)) => value,
            _ => 0.0,
        }
    }

    /// A story state value as text, or the empty string if there is none.
    pub fn get_text(&mut self, conversation: &str, key: &str) -> String {
        match self.variant(conversation, key) {
            Some(LouderVariantArm::string(value)) => self.text(value),
            _ => String::new(),
        }
    }

    /// Write an integer story state value.
    pub fn set_int(&mut self, conversation: &str, key: &str, value: i32) {
        let name = self.declare(conversation);
        let state = self.declare(key);
        let variant = unsafe { ffi::make_variant_int(value) };
        unsafe { ffi::set_story_state(self.ctx, name, state, variant) };
    }

    /// Write a float story state value.
    pub fn set_float(&mut self, conversation: &str, key: &str, value: f32) {
        let name = self.declare(conversation);
        let state = self.declare(key);
        let variant = unsafe { ffi::make_variant_float(value) };
        unsafe { ffi::set_story_state(self.ctx, name, state, variant) };
    }

    /// Write a text story state value, stored as the interned name.
    pub fn set_name(&mut self, conversation: &str, key: &str, value: &str) {
        let name = self.declare(conversation);
        let state = self.declare(key);
        let text = self.declare(value);
        let variant = unsafe { ffi::make_variant_string(text) };
        unsafe { ffi::set_story_state(self.ctx, name, state, variant) };
    }

    /// The whole story state as a JSON document.
    pub fn state_json(&mut self, conversation: &str) -> Result<String, Failure> {
        let name = self.declare(conversation);
        let raw = read_into(
            |buffer, size| unsafe { ffi::read_state_json(self.ctx, name, buffer, size) as isize },
            JSON_BUFFER,
        )?;
        Ok(String::from_utf8_lossy(&raw).into_owned())
    }

    /// The story state and every actor, as a JSON document.
    pub fn snapshot_json(&mut self, conversation: &str) -> Result<String, Failure> {
        let name = self.declare(conversation);
        let raw = read_into(
            |buffer, size| unsafe { ffi::read_snapshot_json(self.ctx, name, buffer, size) as isize },
            JSON_BUFFER,
        )?;
        Ok(String::from_utf8_lossy(&raw).into_owned())
    }

    // --- iteration ---

    /// Every queued event, oldest first, leaving the queue alone.
    ///
    /// Every one of the cursors in the header has the same shape: a `start`, the
    /// thing it returns, and a sentinel for the end of the walk. Here they are
    /// iterators, so a walk reads as a loop and the caller cannot forget to
    /// advance.
    pub fn events(&mut self) -> impl Iterator<Item = Event> + '_ {
        EventIter { engine: self, index: 0 }
    }

    /// Every queued event, oldest first, and empty the queue.
    pub fn drain_events(&mut self) -> Vec<Event> {
        let out: Vec<Event> = self.events().collect();
        unsafe { ffi::flush_events(self.ctx) };
        out
    }

    /// Every hub in a channel, in ascending position order.
    pub fn hubs(&mut self, conversation: &str, channel: &str) -> Vec<i32> {
        let name = self.declare(conversation);
        let on = self.declare(channel);
        let mut out = Vec::new();
        let mut start: i32 = -1;
        loop {
            let found = unsafe { ffi::next_hub_by_channel(self.ctx, name, on, start) };
            if found == -1 {
                return out;
            }
            out.push(found);
            start = found;
        }
    }

    /// Every actor speaking as a speaker.
    pub fn actors_speaking_as(&mut self, conversation: &str, speaker: &str) -> Vec<i32> {
        let name = self.declare(conversation);
        let as_name = self.declare(speaker);
        let mut out = Vec::new();
        let mut start: i32 = LOUDER_ACTOR_NONE;
        loop {
            let found = unsafe { ffi::next_actor_speaking_as(self.ctx, name, as_name, start) };
            if found == LOUDER_ACTOR_NONE {
                return out;
            }
            out.push(found);
            start = found;
        }
    }

    /// Every speaker an actor speaks as.
    pub fn speakers_of(&mut self, conversation: &str, actor: i32) -> Vec<String> {
        let name = self.declare(conversation);
        let mut out = Vec::new();
        let mut start = LOUDER_NAME_NONE;
        loop {
            let found = unsafe { ffi::next_actor_speak_as(self.ctx, name, actor, start) };
            if found == LOUDER_NAME_NONE {
                return out;
            }
            out.push(self.text(found));
            start = found;
        }
    }

    /// Every tag on a hub.
    pub fn tags_of(&mut self, conversation: &str, position: i32) -> Vec<String> {
        let name = self.declare(conversation);
        let mut out = Vec::new();
        let mut start = LOUDER_NAME_NONE;
        loop {
            let found = unsafe { ffi::next_hub_tag(self.ctx, name, position, start) };
            if found == LOUDER_NAME_NONE {
                return out;
            }
            out.push(self.text(found));
            start = found;
        }
    }

    /// Every speaker listening to another.
    pub fn speakers_listening_to(&mut self, speaker: &str) -> Vec<String> {
        let name = self.declare(speaker);
        let mut out = Vec::new();
        let mut start = LOUDER_NAME_NONE;
        loop {
            let found = unsafe { ffi::get_next_speaker_listen_to(self.ctx, name, start) };
            if found == LOUDER_NAME_NONE {
                return out;
            }
            out.push(self.text(found));
            start = found;
        }
    }

    /// The dialogue rules in effect.
    pub fn rules(&mut self) -> DialogueRules {
        unsafe { ffi::get_dialogue_rules(self.ctx) }
    }

    /// Replace the dialogue rules. They take effect on the next move.
    pub fn set_rules(&mut self, rules: DialogueRules) -> bool {
        unsafe { ffi::set_dialogue_rules(self.ctx, rules) == 0 }
    }

    /// Reach an entry point the safe layer did not wrap.
    ///
    /// Every function in the header is declared in `ffi`; this is a way to call
    /// one from here without the two layers having to agree on how.
    pub unsafe fn call_linkage(&mut self) -> i32 {
        ffi::test_linkage()
    }

    fn event(&mut self, raw: LouderEvent) -> Event {
        let value = match raw.data.arm() {
            Some(EventDataArm::regular_motion(m))
            | Some(EventDataArm::warp(m))
            | Some(EventDataArm::force_motion(m)) => Some(EventValue::Motion {
                source_position: m.source_position,
                target_position: m.target_position,
                choice_index: m.choice_index,
                background: m.background,
            }),
            Some(EventDataArm::choice_appearance(c)) => Some(EventValue::ChoiceAppearance {
                position: c.position,
                choice_index: c.choice_index,
            }),
            Some(EventDataArm::speaker_change(value))
            | Some(EventDataArm::text_change(value))
            | Some(EventDataArm::statement(value))
            | Some(EventDataArm::callback(value))
            | Some(EventDataArm::speaker_added(value))
            | Some(EventDataArm::speaker_removed(value))
            | Some(EventDataArm::conversation_added(value))
            | Some(EventDataArm::conversation_removed(value)) => Some(EventValue::Name(self.text(value))),
            _ => Some(EventValue::Other),
        };
        Event {
            conversation: self.text(raw.conversation),
            channel: self.text(raw.channel),
            actor: raw.actor,
            kind: raw.data.kind().unwrap_or("Unknown"),
            value,
        }
    }
}

impl Drop for Louder {
    fn drop(&mut self) {
        self.close();
    }
}

/// Walks the event queue from the outside.
struct EventIter<'a> {
    engine: &'a mut Louder,
    index: usize,
}

impl Iterator for EventIter<'_> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        let found = unsafe { ffi::get_event(self.engine.ctx, self.index) };
        match found.tag {
            LouderOption_LouderEvent_Tag::Some_LouderEvent => {
                self.index += 1;
                Some(self.engine.event(unsafe { found.body.some }))
            }
            _ => None,
        }
    }
}
'''

    # ------------------------------------------------------------------
    # The rest of the crate
    # ------------------------------------------------------------------

    def render_build_script(self) -> list[str]:
        """A build script that finds the engine where cargo left it.

        Without this, `link` looks for the library beside the crate, and the
        engine is built into its own workspace instead. This walks up from the
        crate to the workspace that holds both and puts its target directory on
        the link path.

        The staging directory is there for a naming disagreement rather than a
        missing file: on Windows the engine leaves an import library called
        `louder_rust.dll.lib`, and the MSVC linker opens `louder_rust.lib`. Both
        directories go on the search path, so whichever file a toolchain is
        looking for, it finds.
        """
        stem = self.library_stem()
        return [
            "// Generated from the C header of the Louder engine.",
            "//",
            "// The engine is built into its own workspace, so its library is not beside",
            "// this crate. The search below walks up from the crate to the workspace",
            "// that holds both and points the linker at wherever cargo left it.",
            "",
            "use std::path::Path;",
            "use std::path::PathBuf;",
            "",
            "fn main() {",
            '    println!("cargo:rerun-if-changed=build.rs");',
            '    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));',
            '    let staging = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("link");',
            '    let _ = std::fs::create_dir_all(&staging);',
            "",
            "    for workspace in manifest.ancestors() {",
            '        let Ok(text) = std::fs::read_to_string(workspace.join("Cargo.toml")) else {',
            "            continue;",
            "        };",
            '        if !text.contains("' + stem + '") {',
            "            continue;",
            "        };",
            "",
            '        for profile in ["release", "debug"] {',
            '            let built = workspace.join("target").join(profile);',
            "            if !built.is_dir() {",
            "                continue;",
            "            }",
            '            stage(&built, &staging, "' + stem + '");',
            '            println!("cargo:rustc-link-search=native={}", built.display());',
            '            println!("cargo:rustc-link-search=native={}", staging.display());',
            "            return;",
            "        }",
            "",
            "        println!(",
            '            "cargo:warning=the engine is not built; run cargo build -p ' + stem + '",',
            "        );",
            "        return;",
            "    }",
            "",
            '    println!("cargo:warning=the engine workspace was not found above this crate");',
            "}",
            "",
            "/// Put the import library in the staging directory under the name the",
            "/// linker asks for, if it is not already called that.",
            "///",
            "/// Windows writes `louder_rust.dll.lib` where MSVC `link` opens",
            "/// `louder_rust.lib`. Everywhere else the file already has the right",
            "/// name and this copies nothing.",
            "fn stage(built: &Path, staging: &Path, stem: &str) {",
            '    let wanted = format!("{stem}.lib");',
            '    for name in [format!("{stem}.dll.lib"), format!("lib{stem}.a"), wanted.clone()] {',
            "        let from = built.join(&name);",
            "        if from.is_file() {",
            '            let _ = std::fs::copy(&from, staging.join(&wanted));',
            "            return;",
            "        }",
            "    }",
            "}",
        ]

    def render_cargo(self) -> list[str]:
        return [
            "# Generated from the C header of the Louder engine.",
            "",
            "# Its own package rather than a member of the engine workspace: it is a host",
            "# of the engine, not a part of it. The empty `[workspace]` stops cargo from",
            "# reaching up and mistaking it for one.",
            "[workspace]",
            "",
            "[package]",
            'name = "louder"',
            'version = "0.1.0"',
            'edition = "2021"',
            'description = "Bindings for the Louder dialogue engine"',
            "",
            "[lib]",
            'name = "louder"',
            'path = "src/lib.rs"',
        ]

    def render_example(self) -> list[str]:
        """A walk through the engine, as a program that can be run.

        `cargo run --example walk` exercises every part of the safe layer
        against the real engine, which is the cheapest way to tell a binding that
        is right from one that merely compiles. The values it asserts are the
        ones the engine's own C++ tests assert, so two languages are held to one
        set of answers.

        The program is written by hand in `runtime/rust`: it is a test, not a
        declaration, and there is nothing in the header to derive it from.
        """
        return self.runtime("rust/walk.rs").read_text(encoding="utf-8").splitlines()

    def render_readme(self) -> list[str]:
        return [
            "# Louder, for Rust",
            "",
            "Generated from the C header of the `louder_rust` engine. Do not edit; run",
            "`python engine/bindings/generate.py` and this directory comes back.",
            "",
            "Two modules:",
            "",
            "- `louder::ffi` is the boundary as the header writes it: `#[repr(C)]`",
            "  types, tagged unions as a tag plus a `union`, and the `extern \"C\"`",
            "  block the engine exports.",
            "- `louder::Louder` is the same engine in Rust terms: names are `&str`",
            "  and `String`, buffers size themselves, and the `next_*` cursors are",
            "  iterators.",
            "",
            "```rust",
            "use louder::{ffi::LOUDER_NAME_NONE, Louder};",
            "",
            "let mut engine = Louder::new()?;",
            "engine.load_config(&std::fs::read_to_string(\"speakers.json\")?)?;",
            "engine.load_conversation(\"chat\", &std::fs::read_to_string(\"chat.json\")?)?;",
            "",
            "let actor = engine.add_actor(\"chat\", 0, &[\"A\"], &[]);",
            "engine.choose(\"chat\", actor, 0)?;",
            "",
            "for event in engine.drain_events() {",
            "    println!(\"{} {:?}\", event.kind, event.value);",
            "}",
            "",
            "let standing = engine.actor_position(\"chat\", actor);",
            "println!(\"{}\", engine.hub_text(\"chat\", standing));",
            "println!(\"{:?}\", engine.state_json(\"chat\"));",
            "```",
            "",
            "## Building",
            "",
            "The `ffi` block links `louder_rust` at build time. `build.rs` finds the",
            "engine's workspace by walking up from this crate and puts its target",
            "directory on the link path, so build the engine first:",
            "",
            "```",
            "cargo build -p louder_rust",
            "```",
            "",
            "## Checking it",
            "",
            "```",
            "cargo run --example walk",
            "```",
            "",
            "The walk asserts the same values the engine's own C++ tests do, through",
            "the same C boundary. If it passes, the bindings are not merely compiling.",
            "",
            "## What was generated",
            "",
            "- " + self.counts(),
            "",
        ]

    def counts(self) -> str:
        shapes = self.shapes()
        return (
            str(shapes["functions"]) + " functions, "
            + str(shapes["structs"]) + " structs, "
            + str(shapes["unions"]) + " tagged unions, "
            + str(shapes["enums"]) + " enums and "
            + str(shapes["constants"]) + " constants"
        )
