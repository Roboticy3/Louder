cd ../../engine
.\build_and_test.ps1
cd ../examples/basic_interaction
cp ../../engine/artifacts/louder_rust.dll louder_rust.dll
cp ../../engine/artifacts/louder_rust.h louder_rust.h

g++ client.c server.c test.c louder_rust.dll