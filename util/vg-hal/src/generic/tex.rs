use std::marker::PhantomData;

pub struct GTexture<B: gfx_hal::Backend> {
    _d: PhantomData<B>,
}
