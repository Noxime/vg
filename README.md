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
* Android (broken)
* iOS (incomplete, no test hardware)
* Nintendo Switch (incomplete, lack of stdlib)

## Supported apis
* OpenGL (enabled default)
* Vulkan (available on windows, linux, android and switch)
* Metal (available on macos and ios)
* DirectX 12 (available on windows **10**)

# Get started
Clone this repo and run
```sh
./configure <target>
```

Where target is one of `linux`, `macos`, `windows`, `android`, `ios` or `switch`. This will generate a makefile, after which you can

```sh
make
# or `make build`, `make run`, `make clean` or `make release`
```

To work on your game, go to `game/` and to work on the engine, go to `engine/`.

Peep into the `target-<target>` folders if you want to learn more about that specific target.

# Dependencies
## Win
**TODO** (sorry)
## Nix
**TODO** (sorry)
## OSX
**TODO** (sorry)
## And
**TODO** (sorry)
## iOS
**TODO** (sorry)
## nNX
**TODO** (sorry)