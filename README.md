# glayout

A 2D painting library written in Rust, running in OpenGL/WebGL environment

**STILL IN EARLY DEVELOPMENT**

## Concept

Glayout is a library that...

* making 2D drawing easy
* compiled to native app with OpenGL ES, or compiled to web app with WebGL
* written in Rust
* has Rust interface, and JavaScript interface (web target only)
* similar to Document Object Model, with simple CSS support

## Status

Still in early development. If you are interested, have a try!

Currently this repo has an example app built in. Compile to native app:

1. Have stable [Rust toolchain](https://rustup.rs/) installed
1. Run with `cargo run`

Compile to web app:

1. Have stable [Rust toolchain](https://rustup.rs/) installed
1. Have [emscripten toolchain](http://kripken.github.io/emscripten-site/) installed
1. Run `npm install`
1. Run `npm run build-asmjs-debug`
