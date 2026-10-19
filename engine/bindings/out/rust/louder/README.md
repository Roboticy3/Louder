# Louder, for Rust

Generated from the C header of the `louder_rust` engine. Do not edit; run
`python engine/bindings/generate.py` and this directory comes back.

Two modules:

- `louder::ffi` is the boundary as the header writes it: `#[repr(C)]`
  types, tagged unions as a tag plus a `union`, and the `extern "C"`
  block the engine exports.
- `louder::Louder` is the same engine in Rust terms: names are `&str`
  and `String`, buffers size themselves, and the `next_*` cursors are
  iterators.

```rust
use louder::{ffi::LOUDER_NAME_NONE, Louder};

let mut engine = Louder::new()?;
engine.load_config(&std::fs::read_to_string("speakers.json")?)?;
engine.load_conversation("chat", &std::fs::read_to_string("chat.json")?)?;

let actor = engine.add_actor("chat", 0, &["A"], &[]);
engine.choose("chat", actor, 0)?;

for event in engine.drain_events() {
    println!("{} {:?}", event.kind, event.value);
}

let standing = engine.actor_position("chat", actor);
println!("{}", engine.hub_text("chat", standing));
println!("{:?}", engine.state_json("chat"));
```

## Building

The `ffi` block links `louder_rust` at build time. `build.rs` finds the
engine's workspace by walking up from this crate and puts its target
directory on the link path, so build the engine first:

```
cargo build -p louder_rust
```

## Checking it

```
cargo run --example walk
```

The walk asserts the same values the engine's own C++ tests do, through
the same C boundary. If it passes, the bindings are not merely compiling.

## What was generated

- 88 functions, 15 structs, 4 tagged unions, 5 enums and 14 constants

