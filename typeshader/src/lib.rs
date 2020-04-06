//! Compile-time verified and translated shaders
#![allow(incomplete_features)]
#![feature(const_generics, const_fn, const_loop, never_type, array_value_iter, type_name_of_val)]

use std::marker::PhantomData;
#[cfg(feature = "derive")]
pub use typeshader_derive::shader;

#[cfg(all(test, feature = "derive"))]
mod tests;

mod params;
pub use params::*;
mod types;
pub use types::*;

#[derive(Debug)]
pub struct Program<U: Uniforms, I: Inputs, O: Outputs> {
    #[allow(dead_code)]
    vertex: Shader,
    #[allow(dead_code)]
    fragment: Shader,
    _uniforms: PhantomData<U>,
    _inputs: PhantomData<I>,
    _outputs: PhantomData<O>,
}

impl<U: Uniforms, I: Inputs, O: Outputs> Program<U, I, O> {
    pub const UNIFORM_COUNT: usize = U::COUNT;
    pub const INPUT_COUNT: usize = I::COUNT;
    pub const OUTPUT_COUNT: usize = O::COUNT;

    pub const fn num_uniforms(&self) -> usize {
        Self::UNIFORM_COUNT
    }

    pub const fn num_inputs(&self) -> usize {
        Self::INPUT_COUNT
    }

    pub const fn num_outputs(&self) -> usize {
        Self::OUTPUT_COUNT
    }

    /// Create a new program out raw shaders
    ///
    /// You generally should NOT call this manually, as this circumvents all type safety
    pub const unsafe fn new(vertex: Shader, fragment: Shader) -> Self {
        Program {
            vertex,
            fragment,
            _uniforms: PhantomData,
            _inputs: PhantomData,
            _outputs: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct Shader {
    #[allow(dead_code)]
    glsl: &'static str,
    #[allow(dead_code)]
    hlsl: &'static str,
    #[allow(dead_code)]
    msl: &'static str,
    #[allow(dead_code)]
    spirv: &'static [u32],
}

impl Shader {
    /// Create a new shader out raw shader data
    ///
    /// You generally should NOT call this manually, as this circumvents source equality validation
    pub const unsafe fn new(
        glsl: &'static str,
        hlsl: &'static str,
        msl: &'static str,
        spirv: &'static [u32],
    ) -> Self {
        Shader {
            glsl,
            hlsl,
            msl,
            spirv,
        }
    }
}
