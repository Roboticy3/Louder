# Louder, from C++

Generated. Do not edit; run `python engine/bindings/generate.py` instead.

## Files

| File | What it is |
| --- | --- |
| `louder.h` | The boundary: every type, constant, and entry point, with the layout pinned by static assertions. |
| `louder.hpp` | The layer to use: an owning context, strings, and a loop for each cursor. |
| `walk.cpp` | A complete example, run against the real library. |

## Using it

```cpp
#include "louder.hpp"

int main() {
    louder::Louder engine;
    const louder::Name conversation = engine.name("scene");

    engine.set_int(conversation, "mood", 7);
    std::puts(engine.state_json(conversation).c_str());
}
```

## The bits worth knowing

- **One file, header-only.** `louder.hpp` includes `louder.h` inside
  `extern "C"`, and there is nothing to link but the engine itself.
- **The context closes itself.** `louder::Louder` is move-only and RAII,
  so there is no `deinit` to forget.
- **Names are integers.** `louder::Name` is an `int32_t` handle. Handles
  come from a hash, so they are not sequential and cannot be guessed.
- **Every cursor is a loop.** `for (const auto hub :
  louder::HubByChannel(...))` is the walk the header would have you write
  out by hand.

