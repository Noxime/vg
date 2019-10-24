use super::Ctx;
use kea::renderer::*;

pub struct Tex {
    tex: webgl_stdweb::WebGLTexture
}

impl kea::renderer::Texture<super::Gfx> for Tex {
    fn new(r: &mut super::Gfx, size: &Size, color: &Color) -> Self {
        let mut buffer = Vec::with_capacity(size[0] * size[1] * 4);
        for _ in 0..(size[0] * size[1]) {
            buffer.push((color[0] * 255.0) as u8);
            buffer.push((color[1] * 255.0) as u8);
            buffer.push((color[2] * 255.0) as u8);
            buffer.push((color[3] * 255.0) as u8);
        }

        let ctx = &r.surface.ctx;
        let tex = ctx.create_texture();
        ctx.bind_texture(Ctx::TEXTURE_2D, tex.as_ref());
        // Note, the third arg (internalFormat) _should_ be glenum but is
        // mistakenly marked as a glint in webgl_stdlib; should I issue about it?
        ctx.tex_image2_d(
            Ctx::TEXTURE_2D,
            0,
            Ctx::RGBA as _,
            size[0] as _,
            size[1] as _,
            0,
            Ctx::RGBA,
            Ctx::UNSIGNED_BYTE,
            Some(buffer.as_slice()),
        );
        // TODO: Add mipmap option?
        ctx.generate_mipmap(Ctx::TEXTURE_2D);
        let tex = tex.unwrap( );
        Tex {
            tex
        }
    }

    fn from_data(r: &mut super::Gfx, size: &Size, data: &Vec<Color>) -> Self {
        unimplemented!()
    }

    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl kea::renderer::Target<super::Gfx> for Tex {
    fn size(&self) -> Size {
        unimplemented!()
    }

    fn set(&mut self, color: &Color) {
        unimplemented!()
    }

    fn draw(&mut self, texture: &Self, shading: &Shading, view: &View, transform: &Transform) {
        unimplemented!()
    }
}
