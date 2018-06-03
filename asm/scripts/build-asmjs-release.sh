#!/bin/sh

EMMAKEN_CFLAGS="--js-library ../lib/bin/interfaces-release.js --pre-js scripts/pre.js --post-js scripts/post.js --llvm-lto 3 -O3 -Os --closure 1" cargo build --target=asmjs-unknown-emscripten --release
