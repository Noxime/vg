pub struct Asset(Box<dyn AssetTrait>);

impl Asset {
    /// Load bytes
    pub async fn load(&self) -> Vec<u8> {
        self.0.load().await
    }
}

#[doc(hidden)]
#[crate::async_trait]
pub trait AssetTrait {
    async fn load(&self) -> Vec<u8>;
}