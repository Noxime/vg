# VG Engine
[![](https://github-actions.40ants.com/Noxime/vg/matrix.svg)](https://owo.codes/noxim/vg)

VG is a cross-platform, cross-api lightweight game engine for prototyping smaller games

# Get started

There are two ways to work on a VG project. The recommended method is to use the cargo-vg command, which will handle building your project correctly and provides useful utilities like automatic reloading. Build times on cargo-vg are in the range of 1-2 seconds.

You can also build your project without cargo-vg, but it means your build times will be significantly larger

## Recommended method
```bash
cd cargo-vg/
cargo install --path .
cd ../test/
cargo vg run
```

## The bad way
```bash
cd test/
cargo run
```

Path | Purpose
-----|--------
`native/` | The VG "host side", which implements all functionality
`rust/` | A VG "client side" Rust crate which exposes the host API to the game
`rust/derive/` | Currently unused leftover
`rust/vg-builder/` | A "magic" crate that automatically builds the game as a proper WASM module when depended on
`rust/vg-types/` | An interface crate which defines the actual API between host and client
`cargo-vg/` | A cargo subcommand for automatically building your game as a WASM module and provides hot-reload
`rust-wasm/` | A modified WASM interpreter used in host when other runtimes are not available
`test/` | An example game used for development
