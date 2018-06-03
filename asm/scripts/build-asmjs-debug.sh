#!/bin/sh

EMMAKEN_CFLAGS="-s EXTRA_EXPORTED_RUNTIME_METHODS=[\'UTF8ToString\'] --js-library ../lib/bin/interfaces-debug.js --pre-js scripts/pre.js --post-js scripts/post.js" cargo build --target=asmjs-unknown-emscripten
