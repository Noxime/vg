use std::ops::Deref;

pub use nanoserde::{DeBin, SerBin};

pub const MOVE_TRIGGER_MAGIC: usize = 0xCAFEBABE;

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

#[derive(SerBin, DeBin, Debug, Clone)]
pub enum Call {
    /// Exit the game
    Exit,

    // Graphics
    /// Draw an asset with specified transform
    Draw(DrawCall),

    // Sound
    Play(PlayCall),

    // Debugging
    /// Print a log message
    Print(String),
}

#[derive(SerBin, DeBin, Debug, Clone)]
pub struct DrawCall {
    pub asset: String,
    pub trans: Transform,
}

#[derive(SerBin, DeBin, Debug, Clone)]
pub struct PlayCall {
    pub asset: String,
    pub looping: bool,
}

#[derive(SerBin, DeBin, Debug)]
pub enum Response {
    Time(f64),
    Up(Key),
    Down(Key),
    Tick,
}

#[derive(SerBin, DeBin, Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Left,
    Right,
    Up,
    Down,
    Space,
    Shift,
    Control,
    Tab,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Escape,
}

#[derive(SerBin, DeBin, Debug, Copy, Clone)]
pub enum Digital {
    Up,
    Down,
    Pressed,
    Raised,
}

impl Deref for Digital {
    type Target = bool;

    fn deref(&self) -> &bool {
        match self {
            Digital::Up | Digital::Raised => &false,
            Digital::Down | Digital::Pressed => &true,
        }
    }
}
