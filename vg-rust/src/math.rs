use std::ops::FnOnce;

use crate::{Vec2, Vec3, Vec4};

/// More general version of Into<f32>
pub trait F32Ext {
    fn to(self) -> f32;
}

macro_rules! impl_f32ext {
    (
        $(
            $ty_: ident
        ),*
    ) => {
        $(
            impl F32Ext for $ty_ {
                fn to(self) -> f32 {
                    self as f32
                }
            }
        )*
    };
}

impl_f32ext!(f32, u8, i8, u16, i16, u32, i32, u64, i64, usize, isize);

/// Builder for Vec2, Vec3 or Vec4
pub struct V;

impl<X: F32Ext, Y: F32Ext> FnOnce<(X, Y)> for V {
    type Output = Vec2;

    extern "rust-call" fn call_once(self, (x, y): (X, Y)) -> Self::Output {
        Vec2::new(x.to(), y.to())
    }
}

impl<X: F32Ext, Y: F32Ext, Z: F32Ext> FnOnce<(X, Y, Z)> for V {
    type Output = Vec3;

    extern "rust-call" fn call_once(self, (x, y, z): (X, Y, Z)) -> Self::Output {
        (x.to(), y.to(), z.to()).into()
    }
}

impl<X: F32Ext, Y: F32Ext, Z: F32Ext, W: F32Ext> FnOnce<(X, Y, Z, W)> for V {
    type Output = Vec4;

    extern "rust-call" fn call_once(self, (x, y, z, w): (X, Y, Z, W)) -> Self::Output {
        (x.to(), y.to(), z.to(), w.to()).into()
    }
}
