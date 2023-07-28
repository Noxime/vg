use std::{path::PathBuf, sync::Arc, time::Duration};

use dashmap::DashMap;
use tokio::task::JoinHandle;

use crate::Assets;

/// Loader implementation using the local filesystem
pub struct FileSource {
    assets: Arc<Assets>,
    pending: DashMap<PathBuf, JoinHandle<()>>,
}

impl FileSource {
    pub fn new(assets: Arc<Assets>) -> FileSource {
        FileSource {
            assets,
            pending: DashMap::new(),
        }
    }

    pub async fn drive(&self) -> ! {
        // Loads missing files indefinitely
        let loader = async {
            loop {
                for path in self.assets.missing() {
                    self.pending.entry(path.clone()).or_insert_with(|| {
                        let assets = Arc::clone(&self.assets);
                        tokio::spawn(async move {
                            let bytes = tokio::fs::read(&path).await.unwrap();
                            assets.update(path, bytes);
                        })
                    });
                }
                self.assets.wait_missing().await;
            }
        };

        // Garbage collect dead tasks every 1s
        let gc = async {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                self.pending.retain(|_, task| !task.is_finished());
                interval.tick().await;
            }
        };

        tokio::join!(loader, gc).0
    }
}
