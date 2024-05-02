//! Multi-source, non-blocking, hot-reloading asset server implementation

mod asset;
mod assets;
mod file;

pub use asset::{Asset, AssetKind, BinAsset};
pub use assets::Assets;
pub use file::FileSource;
