cd louder_rust
cargo build
cbindgen --config cbindgen.toml --crate louder_rust --output target/debug/louder_rust.h
cd ..

cp louder_rust/target/debug/louder_rust.dll artifacts/louder_rust.dll
cp louder_rust/target/debug/louder_rust.h artifacts/louder_rust.h

cp louder_rust/target/debug/louder_rust.dll tests/louder_rust.dll
cp louder_rust/target/debug/louder_rust.h tests/louder_rust.h

# Compile the C++ tests against the freshly generated header and DLL, then run
# them. The header is an output, so the tests always exercise the current ABI.
g++ -std=c++17 -I artifacts tests/engine_test.cpp -o artifacts/engine_test.exe artifacts/louder_rust.dll
if (!$?) { throw "failed to compile the engine tests" }

# The DLL sits beside the test binary, which is where Windows looks first.
& artifacts/engine_test.exe
if ($LASTEXITCODE -ne 0) { throw "engine tests failed with exit code $LASTEXITCODE" }