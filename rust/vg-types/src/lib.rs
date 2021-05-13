pub use nanoserde::{DeBin, SerBin};

type Vec3 = [f32; 3];
type Quat = [f32; 4];

#[derive(SerBin, DeBin, Debug, Copy, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        position: [0.0; 3],
        scale: [1.0; 3],
        rotation: [0.0, 0.0, 0.0, 1.0],
    };
}

#[derive(SerBin, DeBin, Debug)]
pub enum Call {
    /// Exit the game
    Exit,

    // Graphics
    /// Finish a single frame of the game and return to host
    Present,
    /// Draw an asset with specified transform
    Draw {
        asset: String,
        trans: Transform,
    },
}
