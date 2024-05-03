#!/bin/bash

# clear previous output files
rm -f *.dylib
rm -f hello.py

# rebuild and generate sxsdk.py
cargo build --features ffi --release
cargo run --features ffi --bin uniffi-bindgen generate --library target/release/libhello.dylib -l python -o ./
cp target/release/libhello.dylib ./

# run python test
# Because generating binary of iOS and Android depends on the system environment, it is difficult to do automated testing.
# In order to do lightweight testing, python is chosen here for validating.
python3 test.py "$@"
status=$?
exit $status
