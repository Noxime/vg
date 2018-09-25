# Kea Engine
Kea engine is a cross-platform, cross-api lightweight game engine for 2D 
games. It is mostly written to support my own game projects, but it 
might be a useful start for you to learn how to port to different 
platforms or how to design a somewhat-functional engine in Rust. 
**Please note:** I am not a professional engine developer, so my code is 
not the perfect.

## Supported platforms
* Linux (`cargo build`)
* Windows (`cargo build`)
* MacOS (`cargo build`)
* Android (see [**Building on android**](#building-on-android))
* iOS (see [**Building on iOS**](#building-on-ios))

## Supported apis
* OpenGL (available on all platforms)
* Vulkan (default on desktop & android)
* Metal (default on apple systems)
* DirectX 12 (disabled by default, only available on Windows 10)

To configure what backends are available, see [**Build features**](#build-features)

### Build features
Disable default features with
```toml
[dependencies.kea]
default-features = false
features = [ <your features> ]
```

* `backend-gl` Enable OpenGL
* `backend-vk` Enable Vulkan
* `backend-mt` Enable Metal
* `backend-dx` Enable DirectX 1


### Building on android
### Building on iOS