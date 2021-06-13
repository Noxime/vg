pub struct Play {
    asset: String,
    looping: bool,
}

pub fn play(asset: impl AsRef<str>) -> Play {
    Play {
        asset: asset.as_ref().into(),
        looping: false,
    }
}

impl Play {
    pub fn loops(&mut self) -> &mut Self {
        self.looping = true;
        self
    }
}

impl Drop for Play {
    fn drop(&mut self) {
        super::call_host(vg_types::Call::Play(vg_types::PlayCall {
            asset: self.asset.clone(),
            looping: self.looping,
        }))
    }
}
