"""Turning the header's names into each language's names.

The engine spells everything in `snake_case`, which most languages agree with
and several do not. Rather than every backend rediscovering how to say
`get_hub_choice_text` in its own idiom, the conversions live here and a backend
says which ones it wants.

Two things get in the way of a mechanical conversion and are handled here
rather than in each backend: names that collide with a keyword once they are
converted, and names that collide with each other once they are not.
"""

from __future__ import annotations

import re
from typing import Iterable

_SPLIT = re.compile(r"[_\s]+")
_HUMPS = re.compile(r"(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])")


def words(name: str) -> list[str]:
    """Break a name into its parts, whichever convention it arrived in.

    `get_hub_text`, `GetHubText` and `get-hub-text` all come out as
    `get`, `hub`, `text`, so a conversion can start from either the header's
    spelling or one the backend produced.
    """
    spaced = _HUMPS.sub(" ", name)
    return [w for w in _SPLIT.split(spaced) if w]


def snake(name: str) -> str:
    return "_".join(w.lower() for w in words(name))


def camel(name: str) -> str:
    parts = words(name)
    if not parts:
        return name
    head, *rest = parts
    return head.lower() + "".join(w[:1].upper() + w[1:].lower() for w in rest)


def pascal(name: str) -> str:
    return "".join(w[:1].upper() + w[1:].lower() for w in words(name))


def upper_snake(name: str) -> str:
    return "_".join(w.upper() for w in words(name))


def screaming(name: str) -> str:
    return "_".join(w.upper() for w in words(name))


def kebab(name: str) -> str:
    return "-".join(w.lower() for w in words(name))


def dotted(name: str) -> str:
    return ".".join(w.lower() for w in words(name))


def keep(name: str) -> str:
    """The name as the header spelled it.

    A backend that writes a C-shaped language wants this: the header is the
    thing being described, and a reader comparing the two should not have to
    work out that `louder_variant` is `LouderVariant`.
    """
    return name


# Keywords and names a backend cannot use as an identifier. Anything listed here
# is suffixed rather than mangled, because a mangled name is one nobody can
# guess when they go looking for it in a header.
RESERVED: dict[str, frozenset[str]] = {
    "python": frozenset(
        """False None True and as assert async await break class continue def del elif else except
        finally for from global if import in is lambda nonlocal not or pass raise return try while
        with yield match case type self cls int float str bool bytes list dict set tuple object None_
        property staticmethod classmethod super""".split()
    ),
    "javascript": frozenset(
        """await break case catch class const continue debugger default delete do else enum export
        extends false finally for function if implements import in instanceof interface let new null
        package private protected public return static super switch this throw true try typeof var
        void while with yield arguments eval undefined NaN Infinity""".split()
    ),
    "typescript": frozenset(),
    "rust": frozenset(
        """as break const continue crate dyn else enum extern false fn for if impl in let loop match
        mod move mut pub ref return self Self static struct super trait true type unsafe use where
        while async await dyn abstract become box do final macro override priv typeof unsized virtual
        yield try union""".split()
    ),
    "go": frozenset(
        """break case chan const continue default defer else fallthrough for func go goto if import
        interface map package range return select struct switch type var nil true false iota len cap
        make new copy delete append string int int32 int64 uint32 uint64 byte rune float32 float64
        bool error complex real imag any""".split()
    ),
    "csharp": frozenset(
        """abstract as base bool break byte case catch char checked class const continue decimal default
        delegate do double else enum event explicit extern false finally fixed float for foreach goto
        if implicit in int interface internal is lock long namespace new null object operator out
        override params private protected public readonly ref return sbyte sealed short sizeof
        stackalloc static string struct switch this throw true try typeof uint ulong unchecked unsafe
        ushort using virtual void volatile while""".split()
    ),
    "java": frozenset(
        """abstract assert boolean break byte case catch char class const continue default do double
        else enum extends final finally float for goto if implements import instanceof int interface
        long native new package private protected public return short static strictfp super switch
        synchronized this throw throws transient try void volatile while record sealed var""".split()
    ),
    "kotlin": frozenset(
        """as break class continue do else false for fun if in interface is null object package return
        super this throw true try typealias typeof val var when while by catch constructor delegate
        dynamic field file finally get import init param property receiver set setparam where actual
        abstract annotation companion const crossinline data enum expect external final infix inline
        inner internal lateinit noinline open operator out reified sealed suspend tailrec vararg
        field it""".split()
    ),
    "swift": frozenset(
        """associatedtype class deinit enum extension fileprivate func import init inout internal let
        open operator private precedencegroup protocol public rethrows static struct subscript
        typealias var break case continue default do else fallthrough for guard if in repeat return
        switch where while as Any catch false is nil super self Self throw throws true try
        Protocol Type""".split()
    ),
    "zig": frozenset(
        """align allowzero and anyframe anytype asm async await break callconv catch comptime const
        continue defer else enum errdefer error export extern fn for if inline linksection noalias
        nosuspend opaque or orelse packed pub resume return struct suspend switch test threadlocal
        try union unreachable usingnamespace var volatile while""".split()
    ),
    "lua": frozenset(
        """and break do else elseif end false for function goto if in local nil not or repeat return
        then true until while""".split()
    ),
    "ruby": frozenset(
        """BEGIN END alias and begin break case class def defined do else elsif end ensure false for if
        in module next nil not or redo rescue retry return self super then true undef unless until
        when while yield""".split()
    ),
    "php": frozenset(
        """abstract and array as break callable case catch class clone const continue declare default do
        echo else elseif empty enddeclare endfor endforeach endif endswitch endwhile enum extends final
        finally fn for foreach function global goto if implements include include_once instanceof
        insteadof interface isset list match namespace new or print private protected public readonly
        require require_once return static switch throw trait try unset use var while xor yield enum
        int float bool string true false null void iterable object mixed never""".split()
    ),
    "d": frozenset(
        """abstract alias align asm assert auto body bool break byte case cast catch cdouble cent cfloat
        char class const continue creal dchar debug default delegate deprecated do double else enum
        export extern false final finally float for foreach function goto idouble if ifloat immutable
        import in inout int interface invariant is lazy long macro mixin module new nothrow null out
        override package pragma private protected public pure real ref return scope shared short static
        struct super switch synchronized template this throw true try typedef typeid typeof ubyte ucent
        uint ulong union unittest ushort version void wchar while with __gshared __traits __vector
        alias""".split()
    ),
    "nim": frozenset(
        """addr and as asm bind block break case cast concept const continue converter defer discard
        distinct div do elif else end enum except export finally for from func if import in include
        interface is isnot iterator let macro method mixin mod nil not notin object of or out proc ptr
        raise ref return shl shr static template try tuple type using var when while xor yield
        addr asm bind cast concept converter""".split()
    ),
    "elixir": frozenset(
        """after alias and case catch cond def defp defexception defguard defmacro defmodule defp do
        else end fn for if import in not or quote raise receive require reraise rescue try unless
        unquote use when with true false nil""".split()
    ),
    "haskell": frozenset(
        """case class data default deriving do else foreign if import in infix infixl infixr instance
        let module newtype of then type where forall mdo family role pattern static group by using
        qualified as hiding hiding""".split()
    ),
    "ocaml": frozenset(
        """and as assert asr begin class constraint do done downto else end exception external for
        fun function functor if in include inherit initializer land lazy let lor lsl lsr lxor match
        method mod module mutable new object of open or private rec sig struct then to try type val
        virtual when while with""".split()
    ),
    "julia": frozenset(
        """baremodule begin break catch const continue do else elseif end export false finally for
        function global if import let local macro module quote return struct true try using while
        abstract mutable primitive type where in isa Int Float String Bool Nothing Any""".split()
    ),
    "r": frozenset(
        """if else repeat while function for next break TRUE FALSE NULL Inf NaN NA_integer_ NA_real_
        NA_character_ in library require source return""".split()
    ),
    "perl": frozenset(
        """if elsif else unless while until for foreach do last next redo my our local state sub
        package use require no eval die warn print printf return qw qq q qr s tr y m qr""".split()
    ),
    "odin": frozenset(
        """asm auto_cast bit_set break case cast context continue defer distinct do dynamic else enum
        fallthrough for foreign if import map matrix notin or or_else package proc return struct switch
        transmute typeid union using where""".split()
    ),
    "v": frozenset(
        """asm assert atomic as asm break const continue defer else enum extern fn for go if import in
        interface is lock match module mut none or pub return select shared sizeof spawn struct type
        typeof union unsafe volatile""".split()
    ),
    "chapel": frozenset(
        """bool complex imag int op real string then type void and as atomic begin borrow by class
        config const continue defer delete do domain downto else enum except export extern for forall
        foreach forward if implements import in include init inline iteratable label let lifetime local
        module new newtype nil noinit only operator otherwise out override owned param private proc
        prototype public record reduce ref return scan select serializable shared single sparse
        subdomain sync then throw try tuple type union use var view when where while with yield zip""".split()
    ),
    "c": frozenset(
        """auto break case char const continue default do double else enum extern float for goto if
        inline int long register restrict return short signed sizeof static struct switch typedef union
        unsigned void volatile while bool true false""".split()
    ),
}


class Names:
    """A backend's naming rules, applied consistently and without collisions."""

    def __init__(self, language: str, *, type_case=pascal, func_case=snake, field_case=snake,
                 const_case=pascal, arg_case=snake, param_case=snake, suffix: str = "_"):
        self.language = language
        self.type_case = type_case
        self.func_case = func_case
        self.field_case = field_case
        self.const_case = const_case
        self.arg_case = arg_case
        self.param_case = param_case
        self.reserved = RESERVED.get(language, frozenset())
        self.suffix = suffix

    # -- one-off ----------------------------------------------------------

    def type(self, name: str) -> str:
        return self._guard(self.type_case(name), "T")

    def func(self, name: str) -> str:
        return self._guard(self.func_case(name), "F")

    def field(self, name: str) -> str:
        return self._guard(self.field_case(name), "F")

    def const(self, name: str) -> str:
        return self._guard(self.const_case(name), "C")

    def arg(self, name: str, position: int) -> str:
        # A parameter the header left unnamed has no name to convert, so it gets
        # one that says where it sits.
        base = self._guard(self.arg_case(name), "A") if name != "unused" else f"arg{position}"
        if not base or base[0].isdigit():
            base = f"{self.suffix}{base}"
        return base

    def param(self, name: str) -> str:
        return self._guard(self.param_case(name), "P")

    # -- collisions -------------------------------------------------------

    def _guard(self, name: str, prefix: str) -> str:
        return f"{name}{self.suffix}" if name in self.reserved else name

    def unique(self, names: Iterable[str], *, base: str = "value") -> str:
        taken = set(names)
        if base not in taken:
            return base
        index = 2
        while f"{base}{index}" in taken:
            index += 1
        return f"{base}{index}"


def doc_lines(text: str, prefix: str, *, width: int = 92) -> list[str]:
    """Reflow a doc comment, prefixed the way the language spells comments.

    The header carries no prose today, so this is here for the moment it does:
    a binding is the first place a caller meets these entry points, and the
    engine's own descriptions are the ones worth carrying over.
    """
    text = (text or "").strip()
    if not text:
        return []

    lines: list[str] = []
    for paragraph in text.split("\n\n"):
        current = ""
        for word in paragraph.split():
            candidate = f"{current} {word}".strip()
            if len(candidate) + len(prefix) > width and current:
                lines.append(f"{prefix}{current}")
                current = word
            else:
                current = candidate
        if current:
            lines.append(f"{prefix}{current}")
        lines.append(f"{prefix}")
    if lines and lines[-1] == f"{prefix}":
        lines.pop()
    return lines
