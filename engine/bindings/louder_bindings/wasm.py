"""Reading the engine's WebAssembly module.

The JavaScript binding calls the exports of a `.wasm` file directly, with no
glue code, which works because the exports follow the C ABI. That ABI has two
details a binding has to know and cannot infer from the header:

- an aggregate returned by value does not come back in a register, so the
  caller passes a pointer to its own space as the *first* argument and the
  function returns nothing;
- an aggregate passed by value is copied into linear memory by the caller, so
  it arrives as a pointer as well.

Both are read here, out of the module's own type section, rather than worked
out from rules. The module is the authority, and a binding that disagrees with
it is a binding that reads the wrong four bytes.

Everything is checked as it is resolved: the number of arguments has to line up
with what the header declares, and each one has to have the width the C type
implies. A mismatch raises rather than producing a call that reads a float as
an integer.
"""

from __future__ import annotations

from dataclasses import dataclass

from .ir import Abi, Func, Named, Pointer, Scalar, Type

WASM_TYPES = {0x7F: "i32", 0x7E: "i64", 0x7D: "f32", 0x7C: "f64"}

_SCALAR_WASM = {
    "void": (),
    "bool": ("i32",),
    "int8_t": ("i32",),
    "uint8_t": ("i32",),
    "int16_t": ("i32",),
    "uint16_t": ("i32",),
    "int32_t": ("i32",),
    "uint32_t": ("i32",),
    "int": ("i32",),
    "unsigned": ("i32",),
    "int64_t": ("i64",),
    "uint64_t": ("i64",),
    "float": ("f32",),
    "double": ("f64",),
}


class WasmError(Exception):
    """Raised when the module and the header disagree about the boundary."""


@dataclass
class WasmSignature:
    """How one function actually appears in the compiled module."""

    name: str
    params: tuple[str, ...]
    results: tuple[str, ...]
    # The argument that receives an aggregate return, which the caller supplies
    # and the engine fills in. None when the return came back in registers.
    sret: int | None
    # Argument indices whose header parameter is an aggregate passed by value,
    # mapped to the name of the type. Those need a copy in linear memory.
    by_value: dict[int, str]

    @property
    def takes_return_slot(self) -> bool:
        return self.sret is not None

    def arity(self) -> int:
        return len(self.params)


class WasmModule:
    """The parts of a `.wasm` file the generator needs."""

    def __init__(self, data: bytes):
        self.data = data
        self.types: list[tuple[tuple[str, ...], tuple[str, ...]]] = []
        self.functions: list[int] = []
        self.exports: dict[str, tuple[int, int]] = {}
        self.memory_pages: int | None = None
        self._parse()

    # -- parsing ----------------------------------------------------------

    def _u32(self, index: int) -> tuple[int, int]:
        result = 0
        shift = 0
        data = self.data
        while True:
            byte = data[index]
            index += 1
            result |= (byte & 0x7F) << shift
            if not byte & 0x80:
                return result, index
            shift += 7

    def _valtype(self, index: int) -> tuple[str, int]:
        byte = self.data[index]
        if byte in (0x63, 0x64):
            # A typed reference carries its heap type after the tag. None of the
            # engine's own signatures use one, but the module has to be walked
            # past them rather than misread.
            heap = self.data[index + 1]
            name = WASM_TYPES.get(heap, f"ref{heap}")
            return (name + "?" if byte == 0x63 else name), index + 2
        if byte not in WASM_TYPES:
            raise WasmError(f"value type 0x{byte:02x} is not one this reader knows")
        return WASM_TYPES[byte], index + 1

    def _parse(self) -> None:
        data = self.data
        if data[:4] != b"\0asm":
            raise WasmError("this file is not a WebAssembly module")
        index = 8
        while index < len(data):
            section = data[index]
            index += 1
            size, index = self._u32(index)
            end = index + size

            if section == 1:
                count, at = self._u32(index)
                for _ in range(count):
                    at += 1  # the function type, always 0x60
                    nparams, at = self._u32(at)
                    params: list[str] = []
                    for _ in range(nparams):
                        name, at = self._valtype(at)
                        params.append(name)
                    nresults, at = self._u32(at)
                    results: list[str] = []
                    for _ in range(nresults):
                        name, at = self._valtype(at)
                        results.append(name)
                    self.types.append((tuple(params), tuple(results)))

            elif section == 2:
                # Imports are not expected: the engine links nothing. Reading
                # them anyway means a module that grew one is reported rather
                # than silently missing a function.
                count, at = self._u32(index)
                for _ in range(count):
                    for _ in range(2):
                        length, at = self._u32(at)
                        at += length
                    kind = data[at]
                    at += 1
                    if kind == 0:
                        type_index, at = self._u32(at)
                        self.functions.append(type_index)
                    else:
                        raise WasmError("the module imports something, which the engine does not")

            elif section == 3:
                count, at = self._u32(index)
                for _ in range(count):
                    type_index, at = self._u32(at)
                    self.functions.append(type_index)

            elif section == 5:
                count, at = self._u32(index)
                for _ in range(count):
                    flags = data[at]
                    at += 1
                    minimum, at = self._u32(at)
                    if flags & 0x01:
                        at, _ = self._u32(at)
                    self.memory_pages = minimum

            elif section == 7:
                count, at = self._u32(index)
                for _ in range(count):
                    length, at = self._u32(at)
                    name = data[at : at + length].decode("utf-8", "replace")
                    at += length
                    kind = data[at]
                    at += 1
                    slot, at = self._u32(at)
                    self.exports[name] = (kind, slot)

            index = end

    # -- lookup -----------------------------------------------------------

    def signature(self, name: str) -> tuple[tuple[str, ...], tuple[str, ...]] | None:
        entry = self.exports.get(name)
        if entry is None or entry[0] != 0:
            return None
        return self.types[self.functions[entry[1]]]

    def has_memory(self) -> bool:
        entry = self.exports.get("memory")
        return entry is not None and entry[0] == 2

    # -- resolution -------------------------------------------------------

    def resolve(self, abi: Abi) -> list[Func]:
        """Attach the module's own signature to every function it exports."""
        resolved: list[Func] = []
        for func in abi.funcs:
            observed = self.signature(func.name)
            if observed is None:
                raise WasmError(f"the module does not export `{func.name}`")
            params, results = observed
            resolved.append(self._match(func, abi, params, results))

        extra = sorted(name for name, (kind, _) in self.exports.items() if kind == 0)
        missing = sorted(set(extra) - {f.name for f in abi.funcs})
        if missing:
            raise WasmError(
                "the module exports functions the header does not declare: "
                + ", ".join(missing)
            )
        return resolved

    def _match(self, func: Func, abi: Abi, params: tuple[str, ...], results: tuple[str, ...]) -> Func:
        expected = [self._param_wasm(abi, p.type) for p in func.params]
        aggregates = {i for i, p in enumerate(func.params) if abi.is_aggregate(p.type)}

        if len(params) == len(expected):
            sret = None
            shifted = params
        elif len(params) == len(expected) + 1 and results == ():
            # One more argument than the header lists, and nothing comes back:
            # that is the slot an aggregate return is written into. It is the
            # first argument, which is where the LLVM WebAssembly ABI puts it.
            sret = 0
            shifted = params[1:]
        else:
            raise WasmError(
                f"`{func.name}` takes {len(params)} arguments in the module and the "
                f"header declares {len(expected)}"
            )

        if results and abi.is_aggregate(func.ret):
            # A small struct of plain integers does come back in a register, so
            # this is a shape the module may legitimately have; anything wider
            # than a register could not be.
            if len(results) != 1:
                raise WasmError(f"`{func.name}` returns {results}, which no C type produces")

        for index, want in enumerate(expected):
            got = shifted[index]
            if got != want:
                raise WasmError(
                    f"`{func.name}` argument {index} is `{got}` in the module "
                    f"and `{want}` in the header"
                )

        by_value: dict[int, str] = {}
        for index, param in enumerate(func.params):
            if index in aggregates:
                by_value[index] = _aggregate_name(abi, param.type)

        return Func(
            name=func.name,
            ret=func.ret,
            params=func.params,
            doc=func.doc,
            wasm=WasmSignature(func.name, params, results, sret, by_value),
        )

    def _param_wasm(self, abi: Abi, type: Type) -> str:
        if isinstance(type, Pointer):
            return "i32"
        resolved = abi.resolve(type)
        if isinstance(resolved, Scalar):
            widths = _SCALAR_WASM.get(resolved.name)
            if widths is None:
                # `size_t` is `usize`, which is a pointer's width on wasm32.
                if resolved.name in ("size_t", "ptrdiff_t", "intptr_t", "uintptr_t"):
                    return "i32"
                raise WasmError(f"no WebAssembly type for `{resolved.name}`")
            return widths[0] if widths else "void"
        # An aggregate passed by value is copied into linear memory by the
        # caller, so what crosses the boundary is a pointer.
        return "i32"


def _aggregate_name(abi: Abi, type: Type) -> str:
    resolved = abi.resolve(type)
    if isinstance(resolved, Named):
        return resolved.name
    return str(resolved)


def load(path) -> WasmModule:
    with open(path, "rb") as handle:
        return WasmModule(handle.read())
