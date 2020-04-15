use crate::{
    gfx::{Source, Texture, WindowMode},
    *,
};

use gl::WebGLRenderingContext as Ctx;
use stdweb::{traits::*, unstable::TryInto, web, web::html_element::CanvasElement, Value};
use webgl_stdweb as gl;

use std::collections::HashMap;

pub struct Gfx {
    canvas: CanvasElement,
    ctx: Ctx,
    matrix: gl::WebGLUniformLocation,
}

pub struct Tex {
    id: gl::WebGLTexture,
}

impl Gfx {
    pub async fn new(canvas: CanvasElement) -> Gfx {
        // TODO: Load prefs
        canvas.set_width(1280);
        canvas.set_height(720);

        let ctx: Ctx = canvas
            .get_context()
            .expect("No WebGL context, is it supported?");

        let vs = ctx.create_shader(Ctx::VERTEX_SHADER).unwrap();
        ctx.shader_source(
            &vs,
            "
        precision highp float;
        attribute vec4 a_pos;

        uniform mat4 u_matrix;

        varying vec2 v_tex;

        void main() {
            gl_Position = a_pos * u_matrix;
            v_tex = (a_pos.xy * vec2(1, -1)) * 0.5 + 0.5;
        }
        ",
        );
        ctx.compile_shader(&vs);

        if let Value::Bool(false) = ctx.get_shader_parameter(&vs, Ctx::COMPILE_STATUS) {
            panic!(
                "Vertex shader error: {}",
                ctx.get_shader_info_log(&vs).unwrap_or("unknown".into())
            );
        }

        let fs = ctx.create_shader(Ctx::FRAGMENT_SHADER).unwrap();
        ctx.shader_source(
            &fs,
            "
        precision highp float;
        uniform sampler2D u_tex;

        varying vec2 v_tex;

        void main() {
            gl_FragColor = texture2D(u_tex, v_tex);
        }
        ",
        );
        ctx.compile_shader(&fs);

        if let Value::Bool(false) = ctx.get_shader_parameter(&fs, Ctx::COMPILE_STATUS) {
            panic!(
                "Vertex shader error: {}",
                ctx.get_shader_info_log(&fs).unwrap_or("unknown".into())
            );
        }

        let program = ctx.create_program().unwrap();
        ctx.attach_shader(&program, &vs);
        ctx.attach_shader(&program, &fs);
        ctx.link_program(&program);

        if let Value::Bool(false) = ctx.get_program_parameter(&program, Ctx::LINK_STATUS) {
            panic!(
                "Shader error: {}",
                ctx.get_program_info_log(&program)
                    .unwrap_or("unknown".into())
            );
        }

        ctx.use_program(Some(&program));

        // vertex positions
        #[rustfmt::skip]
        let points: Vec<f32> = vec![
            -1.0, -1.0, 0.0, 1.0,
            -1.0,  1.0, 0.0, 1.0,
             1.0, -1.0, 0.0, 1.0,
            -1.0,  1.0, 0.0, 1.0,
             1.0, -1.0, 0.0, 1.0,
             1.0,  1.0, 0.0, 1.0,
        ];

        let a_pos = ctx.get_attrib_location(&program, "a_pos") as u32;
        let pos_buf = ctx.create_buffer();
        ctx.bind_buffer(Ctx::ARRAY_BUFFER, pos_buf.as_ref());
        ctx.buffer_data_1(
            Ctx::ARRAY_BUFFER,
            Some(&web::TypedArray::from(points.as_slice()).buffer()),
            Ctx::STATIC_DRAW,
        );

        ctx.enable_vertex_attrib_array(a_pos);
        ctx.bind_buffer(Ctx::ARRAY_BUFFER, pos_buf.as_ref());
        ctx.vertex_attrib_pointer(a_pos, 4, Ctx::FLOAT, true, 0, 0);

        let matrix = ctx.get_uniform_location(&program, "u_matrix").unwrap();

        ctx.enable(Ctx::BLEND);
        ctx.blend_func(Ctx::SRC_ALPHA, Ctx::ONE_MINUS_SRC_ALPHA);

        if let Some(r) = ctx.get_parameter(Ctx::RENDERER).as_str() {
            info!("Renderer: WebGL (\"{}\")", r);
        }

        Gfx {
            ctx,
            matrix,
            canvas,
        }
    }

    pub fn texture(&self, data: (Vec<Color>, Size)) -> Tex {
        let (data, size) = data;

        // TODO: Float textures (OES_float_texture)
        let mut bytes = Vec::with_capacity(data.len() * 4);
        for p in data {
            bytes.push((p.r * 255.0) as u8);
            bytes.push((p.g * 255.0) as u8);
            bytes.push((p.b * 255.0) as u8);
            bytes.push((p.a * 255.0) as u8);
        }

        let id = self.ctx.create_texture();
        self.ctx.bind_texture(Ctx::TEXTURE_2D, id.as_ref());

        // lol good name
        self.ctx.tex_image2_d(
            Ctx::TEXTURE_2D,
            0,
            Ctx::RGBA as _,
            size.w as _,
            size.h as _,
            0,
            Ctx::RGBA,
            Ctx::UNSIGNED_BYTE,
            Some(bytes.as_slice()),
        );

        self.ctx
            .tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_MAG_FILTER, Ctx::NEAREST as _);
        self.ctx
            .tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_MIN_FILTER, Ctx::NEAREST as _);
        self.ctx.tex_parameteri(
            Ctx::TEXTURE_2D,
            Ctx::TEXTURE_WRAP_S,
            Ctx::CLAMP_TO_EDGE as _,
        );
        self.ctx.tex_parameteri(
            Ctx::TEXTURE_2D,
            Ctx::TEXTURE_WRAP_T,
            Ctx::CLAMP_TO_EDGE as _,
        );

        Tex { id: id.unwrap() }
    }

    pub fn fill(&self, color: Color) {
        self.ctx.clear_color(color.r, color.g, color.b, color.a);
        self.ctx.clear(Ctx::COLOR_BUFFER_BIT);
    }

    pub fn draw(&self, texture: &gfx::Texture, instances: &[Mat]) {
        let (w, h) = (
            self.ctx.drawing_buffer_width(),
            self.ctx.drawing_buffer_height(),
        );
        self.ctx.viewport(0, 0, w, h);

        // correct for aspect ratio
        let aspect = match h as f32 / w as f32 {
            x if x < 1.0 => vek::Vec3::new(x, 1.0, 1.0),
            y => vek::Vec3::new(1.0, 1.0 / y, 1.0),
        };
        let view = Mat::identity().scaled_3d(aspect);

        self.ctx
            .bind_texture(Ctx::TEXTURE_2D, Some(&texture.tex.id));
        for mat in instances {
            let mat = mat.transposed() * view;
            self.ctx.uniform_matrix4fv(
                Some(&self.matrix),
                mat.gl_should_transpose(),
                mat.as_col_slice(),
            );
            self.ctx.draw_arrays(Ctx::TRIANGLES, 0, 6);
        }
    }

    pub fn handle(&mut self, _event: &Event) {}

    pub fn present(&self) {
        //webgl already handled
    }
}

impl Vg {
    pub fn title(&mut self, title: impl AsRef<str>) {
        web::document().set_title(title.as_ref());
    }
    pub fn resize(&mut self, _mode: WindowMode) {
        warn!("Window resizing doesn't work on web yet");
    }
}
