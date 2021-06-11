pub struct Play {
    asset: String,
}

pub fn play(asset: impl AsRef<str>) -> Play {
    Play {
        asset: asset.as_ref().into(),
    }
}

impl Drop for Play {
    fn drop(&mut self) {
        super::call_host(vg_types::Call::Play(vg_types::PlayCall {
            asset: self.asset.clone(),
        }))
    }
}
