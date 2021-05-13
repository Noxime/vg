# VG Engine

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