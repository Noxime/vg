pub struct Sfx {

}

impl vg::sfx::SfxTrait for Sfx {
    fn audio(&self, _s: Box<dyn vg::sfx::Source>) -> vg::sfx::Audio {
        unimplemented!()
    }
}