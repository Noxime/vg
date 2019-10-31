use crate::Time;

pub struct Asset(#[doc(hidden)] Box<dyn AssetTrait>);

impl Asset {
    /// Load bytes
    pub async fn load(&self) -> Vec<u8> {
        self.0.load().await
    }

    /// When was the file changed or created
    pub async fn changed(&self) -> Time {
        self.0.changed().await
    }
}

#[doc(hidden)]
#[crate::async_trait]
pub trait AssetTrait {
    async fn load(&self) -> Vec<u8>;
    async fn changed(&self) -> Time;
}
