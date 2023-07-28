use std::{
    path::Path,
    sync::{mpsc, Arc},
};

use crate::Assets;

/// Asset of type T. Created from `Assets`
pub struct Asset<T: AssetKind> {
    eraser: mpsc::Receiver<()>,
    data: T::Data,
    value: Option<T>,
}

impl<T: AssetKind> Asset<T> {
    pub(crate) fn new(assets: &Arc<Assets>, path: &Path) -> Asset<T> {
        Asset {
            value: None,
            eraser: assets.subscribe_eraser(path),
            data: T::new(&assets, path),
        }
    }

    /// Access the value if present
    pub fn get(&mut self) -> Option<&mut T> {
        // Invalidate the inner value. On a loop so multiple erase requests don't
        // stay pending
        // TODO: Perhaps Arc<AtomicBool> is more suitable?
        loop {
            match self.eraser.try_recv() {
                Ok(()) => {
                    self.value = None;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => unreachable!("Lost track of asset source"),
            }
        }

        // Try populate value
        if self.value.is_none() {
            self.value = T::produce(&mut self.data);
        }

        self.value.as_mut()
    }
}

/// Trait implemented by asset types
pub trait AssetKind: Sized {
    /// Internal data used by this asset type. Often is an another Asset<_>
    /// value
    type Data;
    /// Initialize internal data needed for this asset type
    fn new(assets: &Arc<Assets>, path: &Path) -> Self::Data;
    /// Initialize the value if possible. Note that `data` is not reset between
    /// calls
    fn produce(data: &mut Self::Data) -> Option<Self>;
}

/// Simple byte asset. Generally the root for most dependency trees
pub struct BinAsset {
    pub bytes: Vec<u8>,
}

impl AssetKind for BinAsset {
    type Data = mpsc::Receiver<Vec<u8>>;

    fn new(assets: &Arc<Assets>, path: &Path) -> Self::Data {
        assets.subscribe_sender(path)
    }

    fn produce(recv: &mut Self::Data) -> Option<Self> {
        let mut res = None;

        // Loop so we only take the latest value
        loop {
            match recv.try_recv() {
                Ok(bytes) => res = Some(BinAsset { bytes }),
                Err(mpsc::TryRecvError::Empty) => break res,
                Err(mpsc::TryRecvError::Disconnected) => unreachable!("Direct producer hung up"),
            }
        }
    }
}
