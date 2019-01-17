# Kea Engine
[![Build Status](https://dev.azure.com/noxim/kea/_apis/build/status/kea)](https://dev.azure.com/noxim/kea/_build/latest?definitionId=3)

Kea engine is a cross-platform, cross-api lightweight game engine for 2D 
games. It is mostly written to support my own game projects, but it 
might be a useful start for you to learn how to port to different 
platforms or how to design a somewhat-functional engine in Rust. 
**Please note:** I am not a professional engine developer, so my code is 
not the perfect.

## Supported platforms
* Linux
* Windows
* MacOS
* Android
* iOS (borked, GL context lost on launch)
* Nintendo Switch (incomplete, only debugging software renderer)

## Api status
|       |Software|OpenGL|Vulkan|DirectX 12|Metal|
|-------|--------|------|------|----------|-----|
|Linux  |No      |Yes   |WIP   |No        |No   |
|MacOS  |No      |Yes   |No    |No        |WIP  |
|Windows|No      |Yes   |WIP   |WIP       |No   |
|Android|No      |Yes   |WIP   |No        |No   |
|iOS    |No      |Yes   |No    |No        |WIP  |
|Switch |Yes     |WIP   |WIP   |No        |No   |


# Get started
Clone this repo and `$ cd` into one of the preferred targets, most likely 
`target-desktop` and run `cargo build`. If you use any other target, there is a
makefile in each of the `target-*` folders, use `make build` or `make run`.

All game development happens in `game/`, independent of any targets, and engine
development happens in `engine/`. 

# Dependencies
## Win
All dependencies managed by `cargo`
## Nix
All dependencies managed by `cargo`
## OSX
All dependencies managed by `cargo`
## Drd
Docker: `target-android` contains a dockerfile, which should include everything
needed to build. `make build` should even automatically build the dockerfile for
you, so it should theoretically just be plug-and-play. If not, DM me or make an
issue!
## iOS
**TODO** (sorry)
## nNX
**TODO** (sorry)
