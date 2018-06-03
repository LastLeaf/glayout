#!/bin/sh

EMMAKEN_CFLAGS="-s WASM=0 -s SINGLE_FILE=1 --js-library ../lib/bin/interfaces-release.js --pre-js scripts/pre.js --post-js scripts/post.js --llvm-lto 3 -O3 -Os --closure 1" cargo build --target=asmjs-unknown-emscripten --release
