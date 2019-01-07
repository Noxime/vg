use std::marker::PhantomData;
use super::{Error, GSurface};

pub struct GRenderer<B: gfx_hal::Backend> {
    _d: PhantomData<B>
}

impl<B: gfx_hal::Backend> GRenderer<B> {
    pub fn new() -> Result<Box<Self>, Error> {
        unimplemented!()
    }
}

impl<B: gfx_hal::Backend> GRenderer<B> {
    pub fn surface(&mut self) -> GSurface<B> {
        unimplemented!()
    }
}