//! Types for communicating between vg-runtime and vg-rust
pub use nanoserde::{DeBin, SerBin};

#[derive(SerBin, DeBin, Debug)]
pub enum Request {
    Draw(Draw),
}

#[derive(SerBin, DeBin, Debug)]
pub enum Draw {
    Line {
        color: (f32, f32, f32, f32),
        points: Vec<(f32, f32)>,
    },
}

#[derive(SerBin, DeBin, Debug)]
pub enum Response {
    Empty,
}

macro_rules! def_enum {
    (enum $name: ident { $($variant: ident = $value: expr),* }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(C)]
        pub enum $name {
            $(
                $variant = $value,
            )*
        }

        impl $name {
            pub fn from_raw(v: i32) -> Self {
                match v {
                    $(
                        $value => Self :: $variant,
                    )*
                    _ => panic!("Unknown variant {v}")
                }
            }

            pub fn to_raw(self) -> i32 {
                self as i32
            }
        }
    };
}

def_enum! {
    enum WaitReason {
        Startup = 0,
        Present = 1
    }
}

impl WaitReason {
    pub fn is_present(&self) -> bool {
        *self == WaitReason::Present
    }
}