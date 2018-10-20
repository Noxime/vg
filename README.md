# Kea Engine
[![Build Status](https://dev.azure.com/noxim/kea/_apis/build/status/kea)](https://dev.azure.com/noxim/kea/_build/latest?definitionId=3)

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
* OpenGL (enabled default)
* Vulkan (available on windows, linux and android)
* Metal (available on apple)
* DirectX 12 (available on windows **10**)

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
* `backend-dx` Enable DirectX 12


### Building on android
Android support is somewhat meh right now, but heres how you can try it:
1. `docker run --rm -v "$(pwd):/root/src" -w /root/src tomaka/cargo-apk cargo apk build --no-default-features --features=backend-vk`
2. `adb install target/android-artifacts/app/build/outputs/apk/app-debug.apk`

For some reason `gfx-backend-gl` refuses to compile to android (spirv-cross can't find some c++ stdlib files) so if you want to test, use vulkan
### Building on iOS