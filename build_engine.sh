#!/usr/bin/env bash
# Engine build script for the CI. I work on windows so I use the ps1 scripts
# scattered around for testing locally, but this runs on the workflow.
#
# Builds the engine for whichever host it runs on and stages the results under
# site/<platform>/ in the layout the publish job expects:
#
#   site/windows-x86_64/louder_rust.h, louder_rust.dll
#   site/linux-x86_64/louder_rust.h,   liblouder_rust.so
#   site/macos-x86_64/louder_rust.h,   liblouder_rust.dylib
#
# The header is a build output, so every platform regenerates it rather than
# copying one around; the publish job then checks they all agree.

set -euo pipefail

PROFILE="${PROFILE:-release}"
STAGE="${STAGE:-site}"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace="$repo_root/engine"
stage="$repo_root/$STAGE"

# Matches the platform names in the workflow matrix, so a build job and its
# artifact directory always agree.
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) platform="windows-x86_64" ;;
  Linux)                      platform="linux-x86_64" ;;
  Darwin)                     platform="macos-x86_64" ;;
  *)
    echo "build_engine.sh: unsupported platform: $(uname -s)" >&2
    exit 1
    ;;
esac

case "$platform" in
  windows-*) library="louder_rust.dll" ;;
  linux-*)   library="liblouder_rust.so" ;;
  macos-*)   library="liblouder_rust.dylib" ;;
esac

# Cargo spells the dev profile "dev"; "debug" is what a caller is likely to type.
case "$PROFILE" in
  debug) cargo_profile="dev"; target_dir="debug" ;;
  *)     cargo_profile="$PROFILE"; target_dir="$PROFILE" ;;
esac

# The hosted images already carry Rust, so this is a no-op there and a rescue
# anywhere else.
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal
fi
# shellcheck source=/dev/null
. "$HOME/.cargo/env"

if ! command -v cbindgen >/dev/null 2>&1; then
  cargo install cbindgen --locked
fi

echo "building louder_rust ($PROFILE) for $platform"
cd "$workspace"
# The engine on its own: the C boundary is what this publishes, and the Godot
# adapter in louder_godot is not part of it.
cargo build -p louder_rust --profile "$cargo_profile"
cbindgen --config louder_rust/cbindgen.toml --crate louder_rust --output "target/$target_dir/louder_rust.h"

mkdir -p "$stage"
cp "target/$target_dir/$library" "$stage/$library"
cp "target/$target_dir/louder_rust.h" "$stage/louder_rust.h"

echo "staged for $platform:"
ls -l "$stage"