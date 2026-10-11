# Engine

When modifying the engine, run .\build_and_test.ps1 from the engine/ directory
to validate the behavior of the engine on its own.

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