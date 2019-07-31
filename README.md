# Kea Engine

Kea engine is a cross-platform, cross-api lightweight game engine for 2D 
games. It is mostly written to support my own game projects, but it 
might be a useful start for you to learn how to port to different 
platforms or how to design a somewhat-functional engine in Rust. 
**Please note:** I am not a professional engine developer, so my code is 
not the perfect.

## [Documentation](https://noxim.owo.codes/kea)

## Supported platforms
* Linux
* Windows
* MacOS
* Web (borked)
* Android
* iOS (borked, GL context lost on launch)
* Nintendo Switch (borked, only debugging software renderer)

## Api status
|       |Software|OpenGL|Vulkan|DirectX 12|Metal|
|-------|--------|------|------|----------|-----|
|Linux  |No      |Yes   |WIP   |No        |No   |
|MacOS  |No      |Yes   |No    |No        |WIP  |
|Windows|No      |Yes   |WIP   |WIP       |No   |
|Web    |No      |Yes   |No    |No        |No   |
|Android|No      |Yes   |WIP   |WIP       |No   |
|iOS    |No      |Yes   |No    |No        |WIP  |
|Switch |Yes     |WIP   |WIP   |No        |No   |


# Getting started
Please check out the [documentation](https://noxim.owo.codes/kea) on how to
start your Kea based game project!

# Building and dependencies
## Dsk
On desktop, most dependencies are handled by Cargo, but you need to install
`pkg-config`, `libudev` and `libasound2-dev` on *nix platforms.

`sudo apt install pkg-config libudev libasound2-dev`

(On ubuntu this is `libudev-dev`)
## Drd
Docker: `target-android` contains a dockerfile, which should include everything
needed to build. `make build` should even automatically build the dockerfile for
you, so it should theoretically just be plug-and-play. If not, DM me or make an
issue!
## iOS
iOS builds are really quite borked right now, they build (most of the time) and
dependencies are handled by `Cargo` or `XCode` itself. 

**TODO: _More info_**
## nNX
The Switch build is most hacked-up one, and is a total pain to get working on a
new machine. I probably will never provide a windows build environment for this,
but hopefully at some point I'll have a dockerfile for automatic dependency
installation. 

Essentially, right now you need to have 
[DevKitPro](https://switchbrew.org/wiki/Setting_up_Development_Environment) 
toolchain installed. As my `Makefile` is really dumb right now, you need to
install it to `/opt/devkitpro` and add (at least) `elf2nro` to your `PATH`

**TODO: _More info_**
