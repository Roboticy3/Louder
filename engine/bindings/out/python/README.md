# Louder, for Python

Generated from the C header of the `louder_rust` engine. Do not edit;
run `python engine/bindings/generate.py` and this directory comes back.

Two modules:

- `louder._ffi` is the boundary as the header writes it: `ctypes`
  structures, the constants, and the entry points with their
  signatures attached.
- `louder` is the same engine in Python terms: names are `str`,
  buffers are handled for you, a variant is `int | float | str`, and
  the `next_*` family is a generator.

```python
from louder import Louder

with Louder() as engine:
    engine.load_config(open('speakers.json').read())
    engine.load_conversation('chat', open('chat.json').read())

    actor = engine.add_actor('chat', 0, speak_as=['A'])
    engine.choose('chat', actor, 0)

    for event in engine.drain_events():
        print(event.kind, event.value)

    where = engine.actor_position('chat', actor)
    print(engine.hub_text('chat', where))
    print(engine.get('chat', 'mood'))
```

## Loading the engine

The module looks for `louder_rust.dll`, `liblouder_rust.so` or
`liblouder_rust.dylib` beside the package and then on the platform
search path. Build it with:

```
cargo build -p louder_rust --release
```

Point `LOUDER_LIBRARY` at the file, or pass a path to `Louder(path=...)`,
if it lives somewhere else.

## What was generated

- 88 functions, 15 structs, 4 tagged unions, 5 enums, 1 opaque handle and 14 constants

