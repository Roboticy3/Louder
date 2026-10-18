# Engine

`engine/` is a cargo workspace with two packages:

- `louder_rust` is the engine: a plain Rust library with a `extern "C"`
  boundary, and no dependency on any host language. Its docstrings are the
  authoritative documentation of what the engine does.
- `louder_godot` is the Godot adapter: a GDExtension that depends on
  `louder_rust` and declares Godot classes over its types. It holds no engine
  logic, only the wrappers.

A new adapter for another language is a third package in that workspace,
depending on `louder_rust` the same way. Rust does not allow an `impl` block for
a type outside the crate that defines it, which is why the adapter wraps each
engine type in a newtype of its own rather than extending it.

When modifying the engine, run `.\build_and_test.ps1` from the `engine/`
directory to build `louder_rust` alone, regenerate its C header, and run the C++
tests against it. The example project has its own runner for the Godot side: from
`examples/crpg_playtest`, `.\run_test.ps1 test_names.gd` builds the extension and
checks that every method it registers is reachable under its own name.

# Examples

Examples download an engine binary from the internet alongside a build script, so
they work when separated from this repo. Engine CI publishes those binaries to
<https://roboticy3.github.io/Louder> on every push to `main`.

`examples/basic_interaction/build_and_test.ps1` falls back to a local engine build
when that host is unreachable, which is what keeps it usable offline. Set
`LOUDER_OFFLINE` to skip the network, or `LOUDER_BASE_URL` to point it elsewhere.

# Engine binaries

`build_engine.sh` builds the engine for whichever host it runs on and stages the
results in `site/<platform>/`. Engine CI runs it across Windows, Linux, and macOS
and publishes the result to the `gh-pages` branch, which GitHub Pages serves at the
URL above. The examples fetch `<platform>/louder_rust.h` and their own platform's
library from there.