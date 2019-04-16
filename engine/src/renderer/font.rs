//! A utility for rendering text
//! 
//! You provide a font with a hashmap of characters to textures, and a default
//! texture to use if a character does not exist in the mapping.

use std::collections::HashMap;
use super::{Renderer, Transform, Target};

pub struct Font<R: Renderer> {
    mapping: HashMap<char, R::Texture>,
    default: R::Texture,
}

impl<R: Renderer> Font<R> {
    /// Create a new font from a character map and a default
    pub fn new(mapping: HashMap<char, R::Texture>, default: R::Texture) -> Font<R> {
        Font {
            mapping,
            default,
        }
    }

    /// Render a given string
    pub fn render<F>(&self, string: &str, transform: &Transform, mut f: F)
    where
        F: FnMut(&Transform, &R::Texture)
    {
        let mut transform = transform.clone();

        for c in string.chars() {
            let tex = self.mapping.get(&c).unwrap_or(&self.default);
            let width = tex.size()[0] as f32 / tex.size()[1] as f32;
            f(&transform, tex);
            transform = transform.translate(width * transform.rotation.cos(), width * transform.rotation.sin());
        }
    }

    /// Get the length of the string with kerning applied
    pub fn width(&self, string: &str) -> f32 {
        let mut s = 0.0;
        for c in string.chars() {
            let tex = self.mapping.get(&c).unwrap_or(&self.default);
            let width = tex.size()[0] as f32 / tex.size()[1] as f32;
            s += width;
        }
        s
    }
}