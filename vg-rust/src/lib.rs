#![feature(fn_traits, unboxed_closures)]

mod consts;
mod executor;
mod ffi;
mod math;

pub use consts::*;
pub use executor::{spawn, start, wait, JoinHandle};
pub use math::{F32Ext, V};
use vg_interface::*;

pub use glam::{self, Mat3, Mat4, Vec2, Vec3, Vec4};

/// Register a `Fn() -> impl Future<Output=()>` as the entrypoint for your game
#[macro_export]
macro_rules! main {
    ($func: ident) => {
        fn main() {
            // Start our game state
            $crate::start($func());
        }
    };
}

pub fn line(color: Vec4, points: impl IntoIterator<Item = Vec2>) {
    let Response::Empty = ffi::dispatch(Request::Draw(Draw::Line {
        color: color.into(),
        points: points.into_iter().map(Into::into).collect()
    })) else {
        panic!("line request returned non-empty");
    };
}

/// Present the current frame to the screen, concluding this game tick
pub async fn present() {
    wait(WaitReason::Present).await
}
