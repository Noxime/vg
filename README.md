# Kea Engine

Kea engine is a cross-platform, cross-api lightweight game engine for 2D 
games. It is mostly written to support my own game projects, but it 
might be a useful start for you to learn how to port to different 
platforms or how to design a somewhat-functional engine in Rust. 

## [Documentation](https://noxim.owo.codes/kea)

![Grab](https://i.imgur.com/fFfBMmN.png)
Kea is focused on indie-type small 2D games, with high flexibility. Here is a
game called [Grab](https://noxim.itch.io/grab) that I made for the GMTK Game 
Jam 2019, to test a beta version of Kea.

## Supported platform tiers
1. Windows, MacOS, Linux
2. Android, iOS, Web
3. Xbox One, PS4, Nintendo Switch

Tier one platforms will always be working, tier two are mostly working but
might have some `unimplemented`s or bugs in them. Tier 3 are experiment and
mostly only exist on my harddrive as an experiment in porting Rust.

# Getting started
Please check out the [documentation](https://noxim.owo.codes/kea) on how to
start your Kea based game project!

# Building and dependencies
If you want an easy build experience, look for docker files in each of the
target-* directories. If they do not exist:
## Desktop
On desktop, most dependencies are handled by Cargo, but you need to install
`pkg-config`, `libudev` and `libasound2-dev` on *nix platforms.

(ubuntu) `sudo apt install pkg-config libudev-dev libasound2-dev`

## iOS
iOS builds are really quite borked right now, they build (most of the time) and
dependencies are handled by `Cargo` or `XCode` itself. 

**TODO: _More info_**
## Switch
The Switch build is most hacked-up one, and is a total pain to get working on a
new machine. I probably will never provide a windows build environment for this,
but hopefully at some point I'll have a dockerfile for automatic dependency
installation. 

Essentially, right now you need to have 
[DevKitPro](https://switchbrew.org/wiki/Setting_up_Development_Environment) 
toolchain installed. As my `Makefile` is really dumb right now, you need to
install it to `/opt/devkitpro` and add (at least) `elf2nro` to your `PATH`

**TODO: _More info_**
