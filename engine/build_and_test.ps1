cd louder_rust
cargo build -p louder_rust
cd ..

Copy-Item louder_rust/target/debug/louder_rust.dll tests/louder_rust.dll

cd tests
gcc test.cpp louder_rust.dll -o test.exe
.\test.exe
cd ..