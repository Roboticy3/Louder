# Louder, for JavaScript

Generated from the C header of the `louder_rust` engine. Do not edit; run
`python engine/bindings/generate.py` and this directory comes back.

There is no glue code. The engine is built to WebAssembly, the module is
instantiated directly, and its exports are called with the arguments its own
type section says they take. The same module runs in Node, in Deno and in a
browser.

```js
import { readFile } from 'node:fs/promises';
import { Louder } from './louder/index.js';

const engine = await Louder.load(await readFile('louder_rust.wasm'));

engine.loadConfig(await readFile('speakers.json', 'utf8'));
engine.loadConversation('chat', await readFile('chat.json', 'utf8'));

const actor = engine.addActor('chat', 0, { speakAs: ['A'] });
engine.choose('chat', actor, 0);

for (const event of engine.drainEvents()) {
  console.log(event.kind, event.value);
}

for (const hub of engine.hubByChannel('chat', 'main')) {
  console.log(hub, engine.hubText('chat', hub));
}

console.log(engine.get('chat', 'mood'));
engine.close();
```

## Building the engine for WebAssembly

```
cargo build -p louder_rust --release --target wasm32-unknown-unknown
```

The module lands under `engine/target/wasm32-unknown-unknown/` and is copied
beside these files.

## The layers

- `louder/raw.js` is the boundary as the module exports it: `SIZE`,
  `OFFSET`, a codec per declared type, and one method per entry point.
- `louder/index.js` is the same engine in JavaScript terms.
- `louder/index.d.ts` is the same in TypeScript terms, and is the
  TypeScript binding.
- `louder/memory.js` owns the scratch memory that both shapes the C ABI
  takes on WebAssembly need. It grows the module's memory and takes the
  pages that come back, which the engine's own allocator never sees.

## What was generated

- 88 functions, 15 structs, 4 tagged unions, 5 enums and 14 constants

