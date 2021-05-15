use std::{collections::HashMap, path::PathBuf};

use anymap::AnyMap;
use tracing::{debug, error, trace, warn};

pub struct Assets {
    paths: Vec<PathBuf>,
    cache: AnyMap,
    defaults: AnyMap,
}

type CacheOf<T> = HashMap<PathBuf, Cached<T>>;

pub struct Cached<T> {
    value: T,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Load for Image {
    fn load(bytes: Vec<u8>) -> Self {
        let image = image::load_from_memory(&bytes).unwrap();
        let image = image.into_rgba8();
        Image {
            width: image.width() as _,
            height: image.height() as _,
            data: image.into_raw(),
        }
    }

    fn placeholder() -> Self {
        Image {
            width: 5,
            height: 5,
            data: vec![0xFF, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x7F].repeat(20),
        }
    }
}

pub trait Load {
    fn load(bytes: Vec<u8>) -> Self;
    fn placeholder() -> Self;
}

impl Assets {
    pub fn new() -> Assets {
        let mut a = Assets {
            paths: vec!["assets/".into()],
            cache: AnyMap::new(),
            defaults: AnyMap::new(),
        };

        a.fix();

        debug!("Asset search paths: {:?}", a.paths);

        a
    }

    fn fix(&mut self) {
        puffin::profile_function!();

        for path in &mut self.paths {
            match path.canonicalize() {
                Ok(p) => *path = p,
                Err(err) => warn!(
                    "Failed to canonicalize asset search path {}: {}",
                    path.display(),
                    err
                ),
            }
        }
    }

    pub fn load<T: Load + 'static>(&mut self, asset: &str) -> &T {
        puffin::profile_function!();
        trace!("Fetching asset: {}", asset);

        // let asset = PathBuf::from(asset);

        // Start a cache for this file type
        if !self.cache.contains::<CacheOf<T>>() {
            self.cache.insert(CacheOf::<T>::new());
        }

        let map: &mut CacheOf<T> = self.cache.get_mut().unwrap();

        // this is a workaround borrowing lifetimes
        let mut found = false;

        // search through cache
        for search in &self.paths {
            let asset = search.join(asset);

            if map.contains_key(&asset) {
                trace!("Found cached, full path: {}", asset.display());
                found = true;
                break;
            }
        }

        // not in cache, search through filesystem
        if !found {
            debug!("Asset {} not in cache, searching", asset);
            for search in &self.paths {
                let asset = search.join(asset);

                match std::fs::read(&asset) {
                    Ok(bytes) => {
                        let value = T::load(bytes);

                        map.insert(asset.clone(), Cached { value });
                        return &map.get(&asset).unwrap().value;
                    }
                    Err(err) => {
                        trace!("Search failed: {}", err)
                    }
                }
            }
        } else {
            for search in &self.paths {
                let asset = search.join(asset);

                if let Some(cached) = map.get(&asset) {
                    return &cached.value;
                }
            }
        }

        error!("Asset not found in cache nor on disk: {}", asset);
        let placeholder = T::placeholder();

        if !self.defaults.contains::<T>() {
            self.defaults.insert(placeholder);
        }

        self.defaults.get().unwrap()
    }
}
