use std::{
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
};

use dashmap::{DashMap, DashSet};
use tokio::sync::Notify;
use tracing::{debug, trace};

use crate::{Asset, AssetKind};

pub struct Assets {
    paths: DashMap<PathBuf, FileData>,
    missing: DashSet<PathBuf>,
    notify: Notify,
}

/// Replaces backslashes with frontslashes
fn normalize_path(path: &Path) -> PathBuf {
    let string = path.to_string_lossy().replace("\\", "/");
    PathBuf::from(string)
}

impl Assets {
    pub fn new() -> Arc<Assets> {
        Arc::new(Assets {
            paths: Default::default(),
            missing: Default::default(),
            notify: Notify::new(),
        })
    }

    /// Access asset, returning None if not yet available
    pub fn get<T: AssetKind>(self: &Arc<Self>, path: impl AsRef<Path>) -> Asset<T> {
        let path = normalize_path(path.as_ref());
        debug!(path = ?path, kind = std::any::type_name::<T>(), "Created asset");
        let asset = Asset::new(self, &path);
        self.notify.notify_waiters();
        asset
    }

    /// List every asset path that should be loaded
    pub fn missing(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.missing.iter().map(|item| item.to_path_buf())
    }

    /// Waits until a new file is declared as missing
    pub async fn wait_missing(&self) {
        self.notify.notified().await
    }

    /// Update the file data for a path
    pub fn update(&self, path: impl AsRef<Path>, bytes: Vec<u8>) {
        let path = normalize_path(path.as_ref());
        self.missing.remove(&path);
        self.paths
            .entry(path.clone())
            .or_insert_with(|| FileData::new(path))
            .send(bytes);
    }

    pub(crate) fn subscribe_eraser(&self, path: &Path) -> mpsc::Receiver<()> {
        self.paths
            .entry(path.to_path_buf())
            .or_insert_with(|| FileData::new(path.to_path_buf()))
            .subscribe_eraser()
    }

    pub(crate) fn subscribe_sender(&self, path: &Path) -> mpsc::Receiver<Vec<u8>> {
        self.missing.insert(path.to_path_buf());
        self.paths
            .entry(path.to_path_buf())
            .or_insert_with(|| FileData::new(path.to_path_buf()))
            .subscribe_sender()
    }
}

struct FileData {
    /// TODO: Only really used for traces
    path: PathBuf,
    /// List of all assets that somehow depend on this value, even indirectly
    dependencies: Mutex<Vec<mpsc::Sender<()>>>,
    /// List of assets that directly want the value
    direct: Mutex<Vec<mpsc::Sender<Vec<u8>>>>,
    /// Current value of the file
    contents: Mutex<Option<Vec<u8>>>,
}

impl FileData {
    fn new(path: PathBuf) -> FileData {
        trace!(?path, "New file data");
        FileData {
            path,
            dependencies: Default::default(),
            direct: Default::default(),
            contents: Default::default(),
        }
    }

    /// Notify all dependent assets to clear their inner value, while garbage
    /// collecting dead references
    fn erase(&self) {
        let mut deps = self.dependencies.lock().unwrap();
        deps.retain(|tx| tx.send(()).is_ok());
        trace!(path = ?self.path, len = deps.len(), "Erased dependants");
    }

    /// Notify all value subscribes to a new version of data
    fn send(&self, bytes: Vec<u8>) {
        self.erase();

        // Send new value to every
        let mut subs = self.direct.lock().unwrap();
        subs.retain(|tx| tx.send(bytes.clone()).is_ok());
        trace!(path = ?self.path, len = subs.len(), "Updated dependants");
    }

    /// Subscribe to this files data, returning an eraser channel
    fn subscribe_eraser(&self) -> mpsc::Receiver<()> {
        let (tx, rx) = mpsc::channel();
        let mut deps = self.dependencies.lock().unwrap();
        deps.push(tx);
        rx
    }

    /// Subscribe to a channel that receives latest file values
    fn subscribe_sender(&self) -> mpsc::Receiver<Vec<u8>> {
        let (tx, rx) = mpsc::channel();

        // Send file contents to new subscriber
        {
            let content = self.contents.lock().unwrap();
            if let Some(bytes) = &*content {
                tx.send(bytes.clone()).unwrap();
            }
        }

        // Add to list of subscribers for any future values
        {
            let mut dirs = self.direct.lock().unwrap();
            dirs.push(tx);
        };

        rx
    }
}

impl Drop for FileData {
    fn drop(&mut self) {
        self.erase();
    }
}
