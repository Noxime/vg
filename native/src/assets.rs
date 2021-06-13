use std::{
    fs::File as StdFile,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::Arc,
};

use bytes::BytesMut;
use dashmap::DashMap;
use tokio::{
    fs::{canonicalize, File},
    io::{AsyncRead, AsyncReadExt, AsyncSeekExt},
};
use tracing::{debug, trace, warn};

pub struct Assets {
    paths: Vec<PathBuf>,
    cache: DashMap<PathBuf, Arc<Cache>>,
}

pub struct Cache {
    pub len: usize,
    pub path: PathBuf,
    pub first: BytesMut,
}

impl Cache {
    pub async fn load_all(&self) -> Vec<u8> {
        puffin::profile_function!();
        let mut buf = self.first.to_vec();

        if buf.len() >= self.len {
            return buf;
        }

        let mut file = File::open(&self.path).await.unwrap();

        // skip first n bytes because those are already pre-loaded
        file.seek(SeekFrom::Start(buf.len() as u64)).await.unwrap();
        file.read_to_end(&mut buf).await.unwrap();

        buf
    }

    pub async fn start_read(&self) -> CacheRead {
        let buf = self.first.to_vec();
        let mut file = StdFile::open(&self.path).unwrap();

        // skip first n bytes because those are already pre-loaded
        file.seek(SeekFrom::Start(buf.len() as u64)).unwrap();

        CacheRead {
            buf,
            file,
            cursor: 0,
        }
    }
}

pub struct CacheRead {
    buf: Vec<u8>,
    file: StdFile,
    cursor: usize,
}

impl tokio_io::AsyncRead for CacheRead {}

impl Read for CacheRead {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.cursor < self.buf.len() {
            let len = buf.len().min(self.buf.len() - self.cursor);
            buf[..len].copy_from_slice(&self.buf[self.cursor..][..len]);
            self.cursor += len;
            Ok(len)
        } else {
            // deallocate any unnecessary memory
            self.buf.clear();
            self.buf.shrink_to_fit();

            self.file.read(buf)
        }
    }
}

impl Assets {
    pub fn new() -> Assets {
        let mut a = Assets {
            paths: vec!["assets/".into()],
            cache: DashMap::new(),
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

    pub async fn get(&self, asset: &str) -> Arc<Cache> {
        puffin::profile_function!();
        trace!("Fetching asset: {}", asset);

        // Look for the asset in any of the search paths
        let mut found = None;
        for source in &self.paths {
            let path = source.join(asset);

            // Already cached, just use that one
            if let Some(cache) = self.cache.get(&path) {
                return Arc::clone(&*cache);
            }

            if let Ok(path) = canonicalize(path).await {
                found = Some(path);
                break;
            }
        }

        let path = found.expect("Asset not found in any search path");
        trace!("Loading asset: {}", asset);

        let preload = 16 * 1024; // Preload first 16kb of data
        let mut buf = BytesMut::with_capacity(preload);
        let mut file = File::open(&path).await.unwrap();
        let meta = file.metadata().await.unwrap();

        file.read_buf(&mut buf).await.unwrap();

        self.cache.insert(
            path.clone(),
            Arc::new(Cache {
                first: buf,
                len: meta.len() as _,
                path: path.clone(),
            }),
        );

        Arc::clone(&*self.cache.get(&path).unwrap())
    }
}
