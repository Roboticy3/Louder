cd louder_rust
cargo build -p louder_rust
cbindgen --config cbindgen.toml --crate louder_rust --output target/debug/louder_rust.h
cd ..

cp louder_rust/target/debug/louder_rust.dll tests/louder_rust.dll
cp louder_rust/target/debug/louder_rust.h tests/louder_rust.h

cd tests
g++ test.cpp louder_rust.dll louder_rust.h -o test.exe
.\test.exe
cd ..