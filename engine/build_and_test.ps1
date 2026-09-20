cd louder_rust
cargo build -p louder_rust
cbindgen --config cbindgen.toml --crate louder_rust --output target/debug/louder_rust.h
cd ..

cp louder_rust/target/debug/louder_rust.dll artifacts/louder_rust.dll
cp louder_rust/target/debug/louder_rust.h artifacts/louder_rust.h
