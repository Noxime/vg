use std::marker::PhantomData;

pub struct GSurface<B: gfx_hal::Backend> {
    _d: PhantomData<B>
}

