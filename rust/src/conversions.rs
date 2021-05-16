/// Any type that can be interpreted as a position in space, like a 2 or 3-dimensional vector
pub trait Position {
    fn to_vec3(self) -> [f32; 3];
}

impl<T: Into<f64>> Position for [T; 2] {
    fn to_vec3(self) -> [f32; 3] {
        let [x, y] = self;
        [x.into() as f32, y.into() as f32, 0.0]
    }
}

impl<T: Into<f64>> Position for [T; 3] {
    fn to_vec3(self) -> [f32; 3] {
        let [x, y, z] = self;
        [x.into() as f32, y.into() as f32, z.into() as f32]
    }
}

pub trait Rotation {
    fn to_quat(self) -> [f32; 4];
}

impl<T: Into<f64>> Rotation for T {
    fn to_quat(self) -> [f32; 4] {
        let f = self.into() as f32 / 2.0;
        [0.0, 0.0, f.sin(), f.cos()]
    }
}
