# Louder bindings

Generated from the C header of the `louder_rust` engine: 88 entry points, 15 structs, 4 tagged unions, 5 enums and 14 constants.

Nothing in here is written by hand. Run `python generate.py` from
`engine/bindings/` and every directory below comes back. The engine's
own documentation is carried across, so the description of an entry
point in any language is the description written next to it in Rust.

## By tier

### Solid — complete bindings that are compiled rather than interpreted

- [Go](go/)

## What every binding gets

Every one of these exposes all
88 functions in `louder_rust/src/lib.rs`, and with them the types
those functions take and return:

- tagged unions, as a tag plus a set of named bodies, never as a blob
- bit flags, as the bits the header defines and not as a bare integer
- the constants, with the names the engine gives them

What differs is how much is wrapped on top. A deep binding takes `str`
where the header takes a buffer and a length, hands a variant back as
`int | float | str`, and turns the `next_*` cursor functions into
iterators. A long one is the declarations and nothing else, which is
the honest amount of work a language whose FFI is a foreign-function
declaration can be handed.

## Shared inputs

- `abi/louder_rust.c.h` — the header every binding is derived from
- `abi/louder_rust.wasm` — the engine built for WebAssembly, whose own
  type section is what the JavaScript binding is written against

