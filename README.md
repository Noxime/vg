# vg Engine

vg engine is a cross-platform, cross-api lightweight game engine for 2D 
games

## [Documentation](https://docs.rs/vg)

![Grab](https://i.imgur.com/fFfBMmN.png)
Vg is focused on small 2D games. Here is a
game called [Grab](https://noxim.itch.io/grab) that I made for the GMTK Game 
Jam 2019, to test a beta version of vg.

## Supported platform tiers
1. Windows, MacOS, Linux
2. Android, iOS, Web
3. ~~Xbox One, PS4, Nintendo Switch~~

Tier one platforms will always be working, tier two are mostly working but
might have some `unimplemented`s or bugs in them. Tier 3 are experiment and
mostly only exist on my harddrive as an experiment in porting Rust.

# Getting started
Please check out the [documentation](https://docs.rs/vg)!

```rust
use vg::*;

#[game]
async fn main(mut vg: Vg) {
    vg.title("Hello world!");

    loop {
        while Some(e) = vg.events() {
            if e == Event::Exit { return }
        }

        vg.present().await;
    }
}
```

## Building
### Desktop
`$ cargo build`

### Web
`$ cargo web build`