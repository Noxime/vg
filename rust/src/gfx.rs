use vg_types::Transform;

use crate::{Position, Rotation};

pub struct Draw {
    asset: String,
    transform: Transform,
}

pub fn draw(asset: impl AsRef<str>) -> Draw {
    Draw {
        asset: asset.as_ref().into(),
        transform: Transform::IDENTITY,
    }
}

impl Draw {
    pub fn pos(mut self, pos: impl Position) -> Draw {
        self.transform.position = pos.to_vec3();
        self
    }

    pub fn rot(mut self, rot: impl Rotation) -> Draw {
        self.transform.rotation = rot.to_quat();
        self
    }
}

impl Drop for Draw {
    fn drop(&mut self) {
        super::call_host(vg_types::Call::Draw {
            asset: self.asset.clone(),
            trans: self.transform.clone(),
        })
    }
}
