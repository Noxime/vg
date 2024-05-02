use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::DashMap;
use notify::{recommended_watcher, NullWatcher, RecursiveMode, Watcher};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{error, info, trace};

use crate::Assets;

/// Loader implementation using the local filesystem
pub struct FileSource {
    assets: Arc<Assets>,
    base: PathBuf,
    pending: DashMap<PathBuf, JoinHandle<()>>,
}

impl FileSource {
    /// Create a new file loader with a base offset
    pub async fn run(assets: Arc<Assets>, base: impl Into<PathBuf>) -> ! {
        let base = base.into();
        trace!(?base, "New file source");

        let (tx, rx) = mpsc::unbounded_channel();
        let this = Arc::new(FileSource {
            assets,
            base,
            pending: DashMap::new(),
        });

        info!("Started FileSource");

        let watcher = recommended_watcher(move |event| tx.send(event).unwrap())
            .map::<Box<dyn Watcher + Send>, _>(|w| Box::new(w))
            .unwrap_or(Box::new(NullWatcher));

        #[allow(unreachable_code)] // Not actually unreachable?
        tokio::join!(Arc::clone(&this).loader(watcher), this.reloader(rx)).0
    }

    // Loads missing files indefinitely
    async fn loader(self: Arc<Self>, mut watcher: Box<dyn Watcher + Send>) -> ! {
        loop {
            info!("Started loading");
            for path in self.assets.missing() {
                let real_path = &self.real_path(&path);
                Arc::clone(&self).load(path);

                if let Err(error) = watcher.watch(real_path, RecursiveMode::NonRecursive) {
                    error!(%error, "Failed to watch file");
                }
            }
            self.assets.wait_missing().await;
        }
    }

    /// Garbage collect dead tasks
    fn garbage_collect(&self) {
        self.pending.retain(|_, task| !task.is_finished());
    }

    /// Start loading a file
    fn load(self: Arc<Self>, path: PathBuf) {
        // TODO: This is kind of sus: If two items are loaded in very quick succession and the first load request fails
        // (io error) then the second one will not kick off, even though it should
        self.garbage_collect();

        Arc::clone(&self)
            .pending
            .entry(path.clone())
            .or_insert_with(move || {
                let real_path = self.real_path(&path);
                trace!(?path, "Started load task");

                tokio::spawn(async move {
                    if let Ok(bytes) = tokio::fs::read(&real_path).await {
                        self.assets.update(path, bytes);
                    }
                    self.garbage_collect();
                })
            });
    }

    async fn reloader(
        self: Arc<Self>,
        mut receiver: mpsc::UnboundedReceiver<notify::Result<notify::Event>>,
    ) -> ! {
        loop {
            let event = receiver.recv().await.unwrap();

            let event = match event {
                Ok(e) => e,
                Err(error) => {
                    error!(%error, "File watcher error");
                    continue;
                }
            };

            trace!(?event, "File event");

            // Ignore some events
            if event.kind.is_remove() || event.kind.is_access() {
                continue;
            }

            for full_path in event.paths {
                let path = self.unreal_path(full_path);
                Arc::clone(&self).load(path);
            }
        }
    }

    /// Convert request path into an actual path on the file system
    fn real_path(&self, path: &Path) -> PathBuf {
        self.base.join(path)
    }

    /// Convert an absolute (canonicalized) path to what asset system would see it as
    fn unreal_path(&self, full_path: PathBuf) -> PathBuf {
        let full_path = full_path.canonicalize().unwrap_or(full_path);
        let full_base = self.base.canonicalize().unwrap_or(self.base.to_path_buf());
        let path = full_path
            .strip_prefix(full_base.to_path_buf())
            .unwrap_or(&full_path)
            .to_path_buf();
        trace!(?full_path, ?path, ?full_base, "Unmapped path");
        path
    }
}
