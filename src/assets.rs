use serde::{Deserialize, Serialize};

// use std::hash::Hash;
use std::collections::HashMap;
use std::fs::{read, read_dir};
use std::{
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    id: u64,
    pub(crate) ty: AssetTy,
}

impl Asset {
    pub fn new(path: impl AsRef<Path>) -> Asset {
        let ty = AssetTy::new(path.as_ref());
        let id = fxhash::hash64(path.as_ref());
        Asset { id, ty }
    }

    // Asset referring to an internal sprite mesh
    pub(crate) fn sprite() -> Asset {
        "#sprite".into()
    }

    pub(crate) fn error_tex() -> Asset {
        "#error.png".into()
    }
}

impl Hash for Asset {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.id);
    }
}

impl PartialEq for Asset {
    fn eq(&self, other: &Asset) -> bool {
        self.id == other.id
    }
}

impl Eq for Asset {}

impl<P: AsRef<Path>> From<P> for Asset {
    fn from(path: P) -> Asset {
        Self::new(path)
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub(crate) enum AssetTy {
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

pub(crate) struct AssetLoader {
    cache: HashMap<Asset, PathBuf>,
}

impl AssetLoader {
    pub fn new() -> AssetLoader {
        AssetLoader {
            cache: AssetLoader::read_data_dir()
                .map_err(|e| {
                    log::warn!("Asset directory (data/) error: {}", e);
                    e
                })
                .unwrap_or_default(),
        }
    }

    // recurse all files in data/ and map their asset id's to real paths
    fn read_data_dir() -> std::io::Result<HashMap<Asset, PathBuf>> {
        let mut cache = HashMap::new();

        fn recurse(
            cache: &mut HashMap<Asset, PathBuf>,
            path: impl AsRef<Path>,
        ) -> std::io::Result<()> {
            let mut root = read_dir(path)?;

            while let Some(Ok(file)) = root.next() {
                if file.file_type()?.is_dir() {
                    recurse(cache, file.path())?
                } else {
                    let path = file.path();
                    let stripped = path.strip_prefix("data/").unwrap();

                    log::trace!("Cached asset {:?}", path);
                    let asset = Asset::new(stripped);
                    cache.insert(asset, path);
                }
            }

            Ok(())
        }

        recurse(&mut cache, "data")?;

        Ok(cache)
    }

    pub fn load(&mut self, asset: &Asset) -> std::io::Result<Vec<u8>> {
        let path = if let Some(path) = self.cache.get(asset) {
            path
        } else {
            log::warn!("Asset not found in data cache, reloading...");
            self.cache = Self::read_data_dir()?;
            self.cache.get(asset).ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File {:X} not found in data/", asset.id),
            ))?
        };

        log::info!("Loading {:?}", path);

        read(path)
    }
}
