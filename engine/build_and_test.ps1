cd louder_rust
cargo build -p louder_rust
cd ..

cd tests
gcc test.cpp ../louder_rust/target/debug/louder_rust.dll -o test.exe
.\test.exe
cd ..