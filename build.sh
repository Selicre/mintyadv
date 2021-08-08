#!/bin/bash

set -euxo pipefail

TARGET=wasm32-unknown-unknown
NAME=mintyadv
BINARY=target/$TARGET/release/$NAME.wasm
DIST=www/$NAME.wasm
UNOPT=www/$NAME.unopt.wasm

cargo build --target $TARGET --lib --release
cp $BINARY $DIST
cp $BINARY $UNOPT
wasm-strip $DIST
wasm-opt -o $DIST -Oz $DIST
# wasm2wat $DIST | sed -E 's/\(export "(__data_end|__heap_base)" \([a-z 0-9]*\)\)//' | wat2wasm - -o $DIST

ls -l $DIST
