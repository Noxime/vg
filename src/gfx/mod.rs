// rendering impl

use crate::Size;

mod texture;
pub use texture::{Source, Texture};

#[cfg(target_arch = "wasm32")]
mod webgl;
#[cfg(target_arch = "wasm32")]
pub use webgl::{Gfx, Tex};

#[cfg(not(target_arch = "wasm32"))]
mod wgpu;
#[cfg(not(target_arch = "wasm32"))]
pub use self::wgpu::{Gfx, Tex};

pub enum WindowMode {
    Window,
    Fullscreen,
    Borderless,
}
