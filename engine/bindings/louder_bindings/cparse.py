"""A parser for the C that `cbindgen` writes.

The header is the ABI contract, and reading it directly means the bindings
cannot drift from the engine: change an entry point, rebuild the header, and
every language follows. It is also the only machine-readable description of the
boundary that does not need a nightly toolchain.

The grammar here is the one `cbindgen --lang C` produces, which is a narrow
slice of C: typedefs, tagged unions, plain structs, `enum`s, function
declarations, and a handful of `#define`s. Anything outside that slice raises
`CParseError` rather than being skipped, because a parser that quietly ignores
what it does not understand produces bindings that are wrong in a way that looks
fine.
"""

from __future__ import annotations

import re
from dataclasses import dataclass

from .ir import (
    Abi,
    Alias,
    Array,
    Const,
    Enum,
    Field,
    Func,
    Named,
    Opaque,
    Param,
    Pointer,
    Scalar,
    Struct,
    TaggedUnion,
    Type,
    Variant,
)


class CParseError(Exception):
    """Raised for anything the model does not describe."""


# --------------------------------------------------------------------------
# Tokenizer
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class Token:
    kind: str  # id | num | str | char | punct | pp
    text: str
    line: int


_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
_NUMBER = re.compile(
    r"0[xX][0-9a-fA-F]+[uUlL]*"
    r"|(?:\d+\.\d*|\.\d+|\d+)(?:[eE][-+]?\d+)?[fFlLuU]*"
)
_OPEN = {"(": ")", "{": "}", "[": "]"}
_CLOSE = {")", "}", "]"}

# Longest first, so that `<=` is not read as `<` followed by `=`.
_PUNCT = [
    "...",
    "<<=",
    ">>=",
    "->",
    "++",
    "--",
    "<<",
    ">>",
    "<=",
    ">=",
    "==",
    "!=",
    "&&",
    "||",
    "+=",
    "-=",
    "*=",
    "/=",
    "%=",
    "&=",
    "|=",
    "^=",
    "##",
    *(c for c in "{}[]();:,*&+-/%<>|^~!=?#."),
]


def tokenize(source: str) -> list[Token]:
    tokens: list[Token] = []
    i = 0
    line = 1
    n = len(source)

    while i < n:
        ch = source[i]

        if ch == "\n":
            line += 1
            i += 1
            continue
        if ch in " \t\r\f\v":
            i += 1
            continue

        if source.startswith("/*", i):
            end = source.find("*/", i + 2)
            if end < 0:
                raise CParseError(f"line {line}: an unterminated comment")
            body = source[i + 2 : end]
            # `/**` is how cbindgen spells a doc comment when the header is built
            # with documentation on, and it is the engine's own prose about each
            # entry point. Worth carrying into every binding rather than losing.
            if body.startswith("*") and not body.startswith("*/"):
                tokens.append(Token("doc", _tidy_doc(body[1:]), line))
            line += source.count("\n", i, end)
            i = end + 2
            continue

        if source.startswith("//", i):
            end = source.find("\n", i)
            i = n if end < 0 else end
            continue

        # A preprocessor line is kept whole. `#include` is not interesting,
        # `#define` carries the constants, and `#if` guards an `enum`'s
        # underlying type, which is the same answer whichever branch a compiler
        # takes.
        if ch == "#":
            end = source.find("\n", i)
            tokens.append(Token("pp", source[i : (n if end < 0 else end)].strip(), line))
            i = n if end < 0 else end
            continue

        if ch in "\"'":
            j = i + 1
            while j < n and source[j] != ch:
                if source[j] == "\\":
                    j += 1
                j += 1
            tokens.append(Token("str" if ch == '"' else "char", source[i : j + 1], line))
            i = j + 1
            continue

        match = _IDENT.match(source, i)
        if match:
            tokens.append(Token("id", match.group(0), line))
            i = match.end()
            continue

        match = _NUMBER.match(source, i)
        if match:
            tokens.append(Token("num", match.group(0), line))
            i = match.end()
            continue

        for punct in _PUNCT:
            if source.startswith(punct, i):
                tokens.append(Token("punct", punct, line))
                i += len(punct)
                break
        else:
            raise CParseError(f"line {line}: cannot read `{ch}`")

    return tokens


_BUILTIN_SCALARS = {
    "void",
    "_Bool",
    "bool",
    "char",
    "signed char",
    "unsigned char",
    "short",
    "unsigned short",
    "int",
    "unsigned",
    "unsigned int",
    "long",
    "unsigned long",
    "long long",
    "unsigned long long",
    "float",
    "double",
    "long double",
    "int8_t",
    "uint8_t",
    "int16_t",
    "uint16_t",
    "int32_t",
    "uint32_t",
    "int64_t",
    "uint64_t",
    "size_t",
    "ssize_t",
    "ptrdiff_t",
    "intptr_t",
    "uintptr_t",
    "intmax_t",
    "uintmax_t",
}

_SCALAR_SPELLINGS = {
    "_Bool": "bool",
    "unsigned int": "uint32_t",
    "unsigned": "uint32_t",
    "unsigned char": "uint8_t",
    "unsigned short": "uint16_t",
    "unsigned long": "uint64_t",
    "unsigned long long": "uint64_t",
    "signed char": "int8_t",
    "long long": "int64_t",
    "long double": "double",
}


_TYPE_WORDS = tuple(sorted(_BUILTIN_SCALARS, key=len, reverse=True))


def _tidy_doc(text: str) -> str:
    """Read a Rust doc comment as prose rather than as Markdown source.

    The comment arrives wrapped and marked up, neither of which means anything
    to the language it is about to be pasted into: the continuation lines are
    unwrapped into paragraphs, and intra-doc links are reduced to the name of
    the thing they point at, since a link into the Rust documentation is dead
    weight once it has left Rust.
    """
    lines: list[str] = []
    for raw in text.splitlines():
        stripped = raw.strip()
        if stripped.startswith("*"):
            stripped = stripped.lstrip("*").strip()
        lines.append(stripped)

    paragraphs: list[str] = []
    current: list[str] = []
    for line in lines:
        if line:
            current.append(line)
        elif current:
            paragraphs.append(" ".join(current))
            current = []
    if current:
        paragraphs.append(" ".join(current))

    out = "\n\n".join(paragraphs)
    out = re.sub(r"\[`([^`]*)`\]", r"\1", out)
    out = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", out)
    return out


# --------------------------------------------------------------------------
# Parser
# --------------------------------------------------------------------------


class Parser:
    """Reads one top-level statement at a time out of a token list."""

    def __init__(self, tokens: list[Token], abi: Abi):
        # A doc comment sits in front of the declaration it describes, so it is
        # lifted off here rather than left where the token helpers would step
        # over it. A directive may come first, since `cbindgen` closes its guard
        # between one declaration and the next.
        self.doc = ""
        index = 0
        while index < len(tokens) and tokens[index].kind in ("pp", "doc"):
            if tokens[index].kind == "doc":
                self.doc = tokens[index].text
            index += 1
        self.tokens = tokens[index:]
        self.pos = 0
        self.abi = abi

    # -- token helpers ----------------------------------------------------

    def _skip_directives(self) -> None:
        # The doc in front of a declaration was already lifted off by
        # `__init__`; anything left is a directive or a doc somewhere the
        # grammar has no place for, and neither may reach it.
        while self.pos < len(self.tokens) and self.tokens[self.pos].kind in ("pp", "doc"):
            self.pos += 1

    def peek(self) -> Token | None:
        self._skip_directives()
        if self.pos >= len(self.tokens):
            return None
        return self.tokens[self.pos]

    def peek_raw(self, ahead: int = 0) -> Token | None:
        index = self.pos
        seen = 0
        while index < len(self.tokens):
            if self.tokens[index].kind != "pp":
                if seen == ahead:
                    return self.tokens[index]
                seen += 1
            index += 1
        return None

    def next(self) -> Token:
        self._skip_directives()
        if self.pos >= len(self.tokens):
            raise CParseError("the header ended in the middle of a declaration")
        token = self.tokens[self.pos]
        self.pos += 1
        return token

    def at_punct(self, text: str) -> bool:
        token = self.peek()
        return token is not None and token.kind == "punct" and token.text == text

    def at_id(self, text: str) -> bool:
        token = self.peek()
        return token is not None and token.kind == "id" and token.text == text

    def at_body_start(self) -> bool:
        """True when the next token opens a brace rather than naming a type.

        `struct Motion motion;` is an elaborated type, while `struct { ... };`
        is a body written out in place, which is what an arm of a tagged union
        looks like.
        """
        return self.at_punct("{")

    def expect_punct(self, text: str) -> None:
        token = self.next()
        if token.kind != "punct" or token.text != text:
            raise CParseError(f"line {token.line}: expected `{text}`, found `{token.text}`")

    def expect_id(self) -> str:
        token = self.next()
        if token.kind != "id":
            raise CParseError(f"line {token.line}: expected a name, found `{token.text}`")
        return token.text

    # -- statements -------------------------------------------------------

    def parse_statement(self) -> None:
        token = self.peek()
        if token is None:
            return
        if token.kind != "id":
            raise CParseError(f"line {token.line}: cannot read a declaration at `{token.text}`")

        if token.text == "typedef":
            self.parse_typedef()
            return

        # `struct Foo { ... };` declares a type, while `struct Foo bar(...);`
        # declares a function that returns one. The two differ only in what
        # follows the name, so the decision is made before either is read.
        if token.text in ("enum", "struct", "union"):
            if self.body_follows():
                self.parse_tagged_declaration()
                return

        self.parse_function()

    def body_follows(self) -> bool:
        """True when a `struct`/`enum`/`union` here declares a type.

        After the keyword and the name comes either a brace (a definition) or a
        semicolon (a forward declaration). A fixed underlying type may sit
        between the name and the brace, as `enum Foo : size_t { ... }` does, so
        the scan steps over the words naming it.
        """
        seen = 0
        index = self.pos
        while index < len(self.tokens) and seen < 2:
            if self.tokens[index].kind not in ("pp", "doc"):
                seen += 1
            index += 1

        while index < len(self.tokens) and self.tokens[index].kind in ("pp", "doc"):
            index += 1
        if index < len(self.tokens) and self.tokens[index].text == ":":
            index += 1
            while index < len(self.tokens) and self.tokens[index].kind in ("id", "punct") and self.tokens[index].text in (
                *_TYPE_WORDS,
                "*",
            ):
                index += 1
                while index < len(self.tokens) and self.tokens[index].kind in ("pp", "doc"):
                    index += 1

        if index >= len(self.tokens):
            return False
        return self.tokens[index].text in ("{", ";")

    def parse_typedef(self) -> None:
        self.next()  # typedef

        # `typedef struct Foo Foo;` with no body: an opaque handle. An `enum`
        # that was written out in full above comes back here as a second name
        # for the same thing, and `typedef struct Foo { ... } Foo;` defines the
        # struct on the way past. All three spellings are the same declaration.
        if self.at_id("struct") or self.at_id("enum") or self.at_id("union"):
            kind = self.take_kind()
            name = self.expect_id()
            self.skip_underlying_type()

            if self.at_punct("{"):
                self.expect_punct("{")
                if kind == "enum":
                    variants = self.parse_enum_body()
                    self.expect_punct("}")
                    self.abi.enums[name] = Enum(name, variants, doc=self.doc)
                else:
                    members = self.parse_body(name, kind)
                    self.expect_punct("}")
                    self.declare_body(name, members, kind)
                name = self.expect_id()

            elif not self.at_punct(";"):
                name = self.expect_id()
                if name in self.abi.enums:
                    return

            self.expect_punct(";")
            if (
                kind == "struct"
                and name not in self.abi.structs
                and name not in self.abi.unions
                and name not in self.abi.enums
            ):
                self.abi.opaques[name] = Opaque(name, doc=self.doc)
            return

        target = self.parse_type()
        name = self.expect_id()
        self.expect_punct(";")

        # `typedef size_t Foo;` sits in the branch of cbindgen's guard that only
        # C17 and older compilers take, and names the same thing as the `enum`
        # above it. The enum wins: it is what carries the variants, and an alias
        # would hide them.
        if name in self.abi.enums or name in self.abi.structs or name in self.abi.unions:
            return
        if isinstance(target, Named) and target.name == name:
            return
        self.abi.aliases[name] = Alias(name, target, doc=self.doc)

    def parse_tagged_declaration(self) -> None:
        kind = self.take_kind()
        name = self.expect_id()
        self.skip_underlying_type()

        if self.at_punct("{"):
            self.expect_punct("{")

            if kind == "enum":
                variants = self.parse_enum_body()
                self.expect_punct("}")
                self.expect_punct(";")
                self.abi.enums[name] = Enum(name, variants, doc=self.doc)
                return

            members = self.parse_body(name, kind)
            self.expect_punct("}")
            self.declare_body(name, members, kind)
            return

        self.expect_punct(";")
        if kind == "enum":
            self.abi.enums.setdefault(name, Enum(name, (), doc=self.doc))
        elif kind == "struct":
            self.abi.opaques.setdefault(name, Opaque(name, doc=self.doc))

    def take_kind(self) -> str:
        token = self.next()
        if token.kind != "id" or token.text not in ("struct", "union", "enum"):
            raise CParseError(f"line {token.line}: expected a tag keyword, found `{token.text}`")
        return token.text

    def skip_underlying_type(self) -> None:
        """Step over the `: size_t` that a fixed underlying type introduces.

        `cbindgen` hides it behind a `__STDC_VERSION__` guard so the header
        stays C17, but both branches agree on the answer, so the guard can go.
        """
        if self.at_punct(":"):
            self.take_punct()
            self.parse_type()

    def take_punct(self) -> str:
        return self.next().text

    def declare_body(self, name: str, members: list[Field], kind: str) -> None:
        """File a `struct`/`union` body as whichever of the two it really is.

        `cbindgen` writes a `#[repr(C)]` enum as a `struct` holding a tag and
        an anonymous union, so the keyword in front of it says nothing about
        which it is. A body that opens with a tag this header already declares
        an `enum` for, and whose remaining member holds the arms, is a tagged
        union; anything else is an ordinary struct.
        """
        tag: Field | None = None
        bodies: Field | None = None
        for member in members:
            if tag is None and bodies is None and isinstance(member.type, Named) and member.type.name in self.abi.enums:
                tag = member
                continue
            bodies = member if bodies is None else bodies

        if tag is not None and bodies is not None and _bare_name(bodies.type) in self.abi.structs:
            holder_name = _bare_name(bodies.type) or ""
            holder = self.abi.structs[holder_name]
            if holder.fields and holder_name in self.abi.union_bodies:
                self.abi.unions[name] = TaggedUnion(
                    name,
                    tag.type,
                    tuple(Variant(f.name, f.type) for f in holder.fields),
                    doc=self.doc,
                )
                return

        if kind == "union":
            raise CParseError(
                f"`{name}` is a union the model does not describe; the engine only "
                "has tagged ones, which cbindgen writes as a struct"
            )
        self.abi.structs[name] = Struct(name, tuple(members), doc=self.doc)

    # -- bodies -----------------------------------------------------------

    def parse_body(self, owner: str, kind: str) -> list[Field]:
        members: list[Field] = []
        while not self.at_punct("}"):
            members.append(self.parse_member(owner, kind))
        return members

    def parse_member(self, owner: str, kind: str) -> Field:
        token = self.peek()

        # `union { struct { T n; }; ... };` — the bodies of a tagged union.
        nxt = self.peek_raw(1)
        if token is not None and token.kind == "id" and token.text == "union" and nxt is not None and nxt.text == "{":
            return self.parse_body_union(owner)

        type = self.parse_type()
        name = self.expect_id()
        type = self.parse_array_suffix(type)
        self.expect_punct(";")
        del kind
        return Field(name, type)

    def parse_body_union(self, owner: str) -> Field:
        """The anonymous union holding every arm of a tagged union.

        Each arm arrives as an anonymous struct wrapping one field named after
        the arm in snake case, so `{"regular_motion": Motion}` is the arm
        `RegularMotion` carrying a `Motion`. The arms are gathered into a
        struct of their own so that a language with no anonymous union has
        something to name; the anonymous structs wrapping them are read and
        discarded, since their shape says nothing the arms do not already.
        """
        self.next()  # union
        self.expect_punct("{")
        name = f"{owner}_Body"
        fields: list[Field] = []

        while not self.at_punct("}"):
            if not self.at_id("struct"):
                token = self.next()
                raise CParseError(f"line {token.line}: an arm of `{owner}` is not a struct")
            self.next()
            self.expect_punct("{")
            members = self.parse_body(name, "struct")
            self.expect_punct("}")
            self.expect_punct(";")
            if len(members) != 1:
                raise CParseError(f"an arm of `{owner}` does not wrap exactly one value")
            fields.append(members[0])

        self.expect_punct("}")
        self.expect_punct(";")

        self.abi.structs[name] = Struct(name, tuple(fields))
        self.abi.union_bodies.add(name)
        return Field("_", Named(name))

    def parse_array_suffix(self, type: Type) -> Type:
        while self.at_punct("["):
            self.take_punct()
            length = _parse_int(self.next().text)
            self.expect_punct("]")
            type = Array(type, length)
        return type

    # -- types ------------------------------------------------------------

    def parse_type(self) -> Type:
        # A leading `const` belongs to whatever is being pointed at, so it is
        # read here and handed to `apply_pointers` rather than stepped over.
        const = False
        while True:
            if self.at_id("const"):
                self.next()
                const = True
                continue
            if self.at_id("volatile"):
                self.next()
                continue
            break

        token = self.peek()
        if token is None:
            raise CParseError("the header ended where a type was expected")

        if token.kind == "id" and token.text in ("struct", "union"):
            kind = self.take_kind()
            name = self.expect_id()
            self.skip_underlying_type()
            if self.at_punct("{"):
                self.expect_punct("{")
                members = self.parse_body(name, kind)
                self.expect_punct("}")
                self.declare_body(name, members, kind)
            return self.apply_pointers(Named(name), const)

        if token.kind == "id" and token.text == "enum":
            self.next()
            name = self.expect_id()
            if self.at_punct("{"):
                self.skip_underlying_type()
                self.expect_punct("{")
                variants = self.parse_enum_body()
                self.expect_punct("}")
                self.abi.enums[name] = Enum(name, variants, doc=self.doc)
            return self.apply_pointers(Named(name), const)

        # A bare scalar, possibly several words: `unsigned long long`.
        words: list[str] = []
        while True:
            token = self.peek()
            if token is None or token.kind != "id":
                break
            if token.text in ("struct", "union", "enum"):
                break
            words.append(self.next().text)
            nxt = self.peek()
            if nxt is None or nxt.kind != "id":
                break
            if " ".join(words + [nxt.text]) in _BUILTIN_SCALARS:
                continue
            break

        if not words:
            raise CParseError(f"line {token.line}: expected a type, found `{token.text}`")

        spelling = " ".join(words)
        if len(words) == 1 and words[0] not in _BUILTIN_SCALARS:
            base: Type = Named(words[0])
        else:
            base = Scalar(_SCALAR_SPELLINGS.get(spelling, spelling))

        return self.apply_pointers(base, const)

    def apply_pointers(self, type: Type, const: bool = False) -> Type:
        """Apply every `*` in front of a type, keeping `const` where it was.

        `const` says the engine will not write through the pointer, which is
        worth keeping: it is the difference between a buffer the caller owns and
        one it is merely lending, and a language that can express the
        difference should.
        """
        while True:
            # A star ends the pointer part of the type. A `const` after one of
            # them qualifies the pointer rather than what it points at, which is
            # what `const char *` means and what C callers expect.
            if self.at_punct("*"):
                self.next()
                type = Pointer(type, const)
                const = False
                if self.at_id("const"):
                    self.next()
                    const = True
                while self.at_id("volatile"):
                    self.next()
                continue
            return type

    def parse_enum_body(self) -> tuple[tuple[str, int | None], ...]:
        variants: list[tuple[str, int | None]] = []
        while not self.at_punct("}"):
            name = self.expect_id()
            value: int | None = None
            if self.at_punct("="):
                self.take_punct()
                value = _parse_int(self.next().text)
            variants.append((name, value))
            if self.at_punct(","):
                self.take_punct()
            elif not self.at_punct("}"):
                token = self.next()
                raise CParseError(f"line {token.line}: expected `,` in an enum body")
        return tuple(variants)

    # -- functions --------------------------------------------------------

    def parse_function(self) -> None:
        head = self.parse_type()
        if self.at_punct("("):
            # A bare return type with no name: not something cbindgen writes.
            raise CParseError("a function declaration has no name")

        name = self.expect_id()
        params = self.parse_param_list()
        self.expect_punct(";")
        # `struct Foo name(...)` arrives as a plain `Foo`, because that is a
        # value being returned; `Foo *name(...)` arrives as a pointer, because
        # that is one. Nothing needs stripping, and stripping would turn the
        # second into the first.
        self.abi.funcs.append(Func(name, head, tuple(params), doc=self.doc))

    def parse_param_list(self) -> list[Param]:
        self.expect_punct("(")
        params: list[Param] = []
        if self.at_punct(")"):
            self.take_punct()
            return params
        if self.at_id("void") and self.peek_raw(1) is not None and self.peek_raw(1).text == ")":
            self.next()
            self.take_punct()
            return params

        while True:
            params.append(self.parse_param())
            if self.at_punct(","):
                self.take_punct()
                continue
            break
        self.expect_punct(")")
        return params

    def parse_param(self) -> Param:
        if self.at_id("void"):
            nxt = self.peek_raw(1)
            if nxt is not None and nxt.text in (")", ","):
                self.next()
                return Param("unused", Scalar("void"))

        type = self.parse_type()
        if self.at_punct(","):
            self.take_punct()
            return Param("unused", type)
        if self.at_punct(")"):
            return Param("unused", type)
        name = self.expect_id()
        return Param(name, self.parse_array_suffix(type))


# --------------------------------------------------------------------------
# Entry point
# --------------------------------------------------------------------------


def _split_statements(tokens: list[Token]) -> list[list[Token]]:
    """Cut the token stream into top-level declarations.

    A declaration ends at the `;` that sits outside every brace, so a struct
    body holding its own `;` stays in one piece.
    """
    statements: list[list[Token]] = []
    current: list[Token] = []
    depth = 0
    for token in tokens:
        if token.kind == "punct":
            if token.text in _OPEN:
                depth += 1
            elif token.text in _CLOSE:
                depth -= 1
            elif token.text == ";" and depth == 0:
                current.append(token)
                statements.append(current)
                current = []
                continue
        current.append(token)
    if any(t.kind not in ("pp", "doc") for t in current):
        statements.append(current)
    return statements


def _parse_defines(tokens: list[Token], abi: Abi) -> None:
    for token in tokens:
        if token.kind != "pp" or not token.text.startswith("#define"):
            continue
        match = re.match(r"#define\s+([A-Za-z_][A-Za-z0-9_]*)\s+(.*)$", token.text, re.DOTALL)
        if not match:
            continue
        const = _parse_define(match.group(1), match.group(2).strip())
        if const is not None:
            abi.consts.append(const)


def _parse_define(name: str, body: str) -> Const | None:
    # `#define ActorFlags_SPECTATOR (ActorFlags){ .bits = (int32_t)1 }`
    compound = re.match(
        r"^\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*\{\s*\.\s*[A-Za-z_][A-Za-z0-9_]*\s*=\s*(.+?)\s*\}$",
        body,
        re.DOTALL,
    )
    if compound:
        value = _parse_number(compound.group(2))
        if isinstance(value, (int, float)):
            return Const(name, Named(compound.group(1)), value)

    value = _parse_number(body)
    if isinstance(value, (int, float)):
        return Const(name, Scalar("int32_t"), value)
    return None


def _parse_number(text: str) -> object:
    text = text.strip()
    while True:
        cast = re.match(r"^\(\s*[A-Za-z_][A-Za-z0-9_]*\s*\)", text)
        if not cast:
            break
        text = text[cast.end() :].strip()
    text = text.rstrip("uUlLfF").strip()
    try:
        if text.lower().startswith(("0x", "-0x")):
            return int(text, 16)
        if re.fullmatch(r"[-+]?\d+", text):
            return int(text)
        if re.fullmatch(r"[-+]?(\d+\.\d*|\.\d+|\d+)([eE][-+]?\d+)?", text):
            return float(text)
    except ValueError:
        pass
    return text


def _parse_int(token: str) -> int:
    value = _parse_number(token)
    if not isinstance(value, int):
        raise CParseError(f"`{token}` should have been a whole number")
    return value


def _bare_name(type: Type) -> str | None:
    while isinstance(type, Pointer):
        type = type.to
    return type.name if isinstance(type, Named) else None


def _is_scalarish(type: Type) -> bool:
    while isinstance(type, Pointer):
        type = type.to
    return isinstance(type, Scalar)


def parse_header(source: str) -> Abi:
    """Read a `cbindgen --lang C` header into an `Abi`."""
    abi = Abi()
    tokens = tokenize(source)
    _parse_defines(tokens, abi)

    for statement in _split_statements(tokens):
        # A statement that is nothing but directives, which happens where
        # cbindgen emits an include between declarations.
        if all(token.kind in ("pp", "doc") for token in statement):
            continue
        Parser(statement, abi).parse_statement()

    abi.link()
    return abi
