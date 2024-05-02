# vg

vg is an indefinitely-work-in-progress game engine focused on rapid prototyping.

# Under construction

## Binaries
* my-game: Example game project
* vg-engine: Main runtime for vg games
* vg-editor: Development tool wrapping vg-engine
* vg-signaling: A matchmaking server

## Libraries
* vg-asset: Asset management library
* vg-interface: Common types between vg-runtime and vg-rust
* vg-network: Network socket and rollback abstraction
* vg-runtime: WebAssembly runtime for state management
* vg-rust: Main interface to the engine, used by games
* vg-scene: Manages the world state

# Scribbles
## Game
my-game is forced-target'ed to wasm32-wasi and is part of the workspace. It will
get automatically built for us, yippee. It only depends on vg-rust and its own,
non-vg dependencies

## Editor
`cargo run` default-runs the editor. Editor is only supported on desktops.

## Engine

### Desktops
```ps1
cargo build -p vg-engine
```

### Web
TODO: High priority

### Android
Requires setting up the Android NDK and cargo apk 0.10. The manifest path is
because cargo apk has trouble with workspaces, apparently. Check the NDK version
is correct and remember --lib so it wont try to make an apk out of a binary
```ps1
$Env:ANDROID_NDK_HOME="T:\Android\ndk\25.2.9519653" 
cargo apk build --lib --manifest-path vg-engine/Cargo.toml
```
