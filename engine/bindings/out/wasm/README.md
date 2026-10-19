# Louder, compiled to WebAssembly

Generated. Do not edit; run `python engine/bindings/generate.py` instead.

## What is here

| File | What it is |
| --- | --- |
| `louder_wasm.h` | The signatures the module exports, as a C header, with every size pinned by a static assertion. |
| `louder_wasm.json` | The same thing for a host that is not C. |
| `louder_rust.wasm` | The engine, built for `wasm32-unknown-unknown`. |

## Loading it

There is no glue code. The module is instantiated and its exports are
called directly.

```javascript
const bytes = await readFile('louder_rust.wasm');
const { instance } = await WebAssembly.instantiate(bytes, {});
const { memory, init_louder, declare_name } = instance.exports;
```

The module exports its memory as `memory`. It imports nothing, because
the engine has nothing to ask the host for.

## The two differences from a plain C header

Everything else about the boundary is the same as the C one. These two
are not, and getting them wrong reads four bytes that mean something
else entirely:

1. **An aggregate returned by value does not come back in a register.**
   For those functions the caller passes a pointer to its own space as
   the *first* argument and the function returns nothing. In the header
   that argument is called `out`; in the JSON it is `returnsThrough`.
2. **An aggregate passed by value arrives as a pointer.** The caller
   copies the struct into linear memory and passes where it went. In the
   header the parameter is marked `by value`; in the JSON it is
   `byValue: true`.

A struct small enough to fit in one register is exempt from the first
rule, which is why `get_actor_flags` returns a plain `i32` while
`get_actor_additional` returns nothing.

## One name difference from the engine's own header

C keeps struct tags, typedef names, and enum constants in one namespace,
and the engine has a variant of `EventData_Tag` called `ChoiceAppearance`
alongside a struct of the same name. That does not compile, so a constant
that would redeclare something else in the file is spelled with the enum's
name in front of it: `EventData_Tag_ChoiceAppearance`. Nothing else is
renamed.

## Where the strings go

Strings cross the boundary as bytes: a `uint8_t*` and a `size_t`. The
engine reads JSON as UTF-8 and stores text as interned names, so a name
crosses as an `int32_t` handle rather than a pointer. Handles come from a
hash, so they are not sequential and cannot be guessed.

## Reading a name back

The engine reports how much text fitted rather than refusing, so the
call reads into a buffer and asks again if it filled:

```javascript
function readName(nameId) {
  for (let size = 4096; ; size *= 4) {
    const at = arena.alloc(size);
    const got = exports.read_name(ctx, nameId, at, size - 1);
    if (got === 0) return '';
    if (got < size - 1) return decoder.decode(memory.buffer, at, got);
  }
}
```

The JavaScript binding in `out/javascript` has that written out, along
with an arena over the module's own linear memory.

