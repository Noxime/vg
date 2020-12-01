use serde::{Serialize, Deserialize};

// use std::hash::Hash;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Asset {
    id: u64,
    ty: AssetTy,
}

impl Asset {
    pub fn new(path: impl AsRef<Path>) -> Asset {
        let ty = AssetTy::new(path.as_ref());
        let id = fxhash::hash64(path.as_ref());
        Asset { id, ty }
    }
}

impl<P: AsRef<Path>> From<P> for Asset {
    fn from(path: P) -> Asset {
        Self::new(path)
    }
}

#[derive(Serialize, Deserialize)]
enum AssetTy {
    Unk,
    Obj,
    Png,
}

impl AssetTy {
    fn new(p: &Path) -> AssetTy {
        match p.extension().and_then(|s| s.to_str()) {
            Some("obj") => AssetTy::Obj,
            Some("png") => AssetTy::Png,
            _ => AssetTy::Unk,
        }
    }
}
