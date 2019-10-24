use std::rc::Rc;
use std::sync::Mutex;
use std::task::Waker;

pub use webgl::WebGLRenderingContext as Ctx;
use webgl_stdweb as webgl;

use stdweb::{console, js};

mod surf;
mod tex;
use surf::Surf;
use tex::Tex;

pub struct Gfx {
    surface: Surf,
}

impl kea::Renderer for Gfx {
    const NAME: &'static str = "WebGL";
    type Texture = Tex;
    type Surface = Surf;

    fn surface(&mut self) -> &mut Surf {
        &mut self.surface
    }
}

impl Gfx {
    pub fn new(
        canvas: stdweb::web::html_element::CanvasElement,
    ) -> (Gfx, Rc<Mutex<Option<std::task::Waker>>>) {
        let ctx: Ctx = canvas.get_context().unwrap();

        // debug clear color
        ctx.clear_color(1.0, 0.7, 1.0, 1.0);
        ctx.clear(Ctx::COLOR_BUFFER_BIT);

        console!(log, "WebGL Context created; ", ctx.get_context_attributes());

        let vs = ctx
            .create_shader(Ctx::VERTEX_SHADER)
            .expect("failed to create vertex shader");
        ctx.shader_source(
            &vs,
            "
            attribute vec4 a_position;
            attribute vec2 a_texcoord;

            uniform mat4 u_matrix;

            varying vec2 v_tex;

            void main() {
                // gl_Position = u_matrix * a_position;
                v_tex = a_texcoord;
                gl_Position = a_position;
            }
        ",
        );
        ctx.compile_shader(&vs);

        if let stdweb::Value::Bool(false) = ctx.get_shader_parameter(&vs, Ctx::COMPILE_STATUS) {
            panic!(
                "Vertex shader error: {}",
                ctx.get_shader_info_log(&vs).unwrap_or("unknown".into())
            );
        }

        let fs = ctx
            .create_shader(Ctx::FRAGMENT_SHADER)
            .expect("failed to create fragment shader");
        ctx.shader_source(
            &fs,
            "
            precision mediump float;

            varying vec2 v_tex;

            uniform sampler2D u_tex;
            
            void main() {
                gl_FragColor = vec4(1, 0, v_tex.s, 1);
            }
        ",
        );
        ctx.compile_shader(&fs);

        if let stdweb::Value::Bool(false) = ctx.get_shader_parameter(&fs, Ctx::COMPILE_STATUS) {
            panic!(
                "Fragment shader error: {}",
                ctx.get_shader_info_log(&fs).unwrap_or("unknown".into())
            );
        }

        // default program
        let prog = ctx
            .create_program()
            .expect("failed to create webgl program");
        ctx.attach_shader(&prog, &vs);
        ctx.attach_shader(&prog, &fs);
        ctx.link_program(&prog);

        if let stdweb::Value::Bool(false) = ctx.get_program_parameter(&prog, Ctx::LINK_STATUS) {
            panic!(
                "Shader error: {}",
                ctx.get_program_info_log(&prog).unwrap_or("unknown".into())
            );
        }

        console!(log, "Created shaders");

        // is this cast safe?
        let a_pos = ctx.get_attrib_location(&prog, "a_position") as u32;
        let a_tex = ctx.get_attrib_location(&prog, "a_texcoord") as u32;
        console!(log, format!("position: {}, texture: {}", a_pos, a_tex));

        let pos_buf = ctx.create_buffer();
        let tex_buf = ctx.create_buffer();
        
        // vertex positions
        ctx.bind_buffer(Ctx::ARRAY_BUFFER, pos_buf.as_ref());

        #[rustfmt::skip]
        let points = vec![
            0.0, 0.0, 0.0, 1.0,
            0.0, 1.0, 0.0, 1.0,
            1.0, 0.0, 0.0, 1.0,

            1.0, 1.0, 0.0, 1.0,
            1.0, 0.0, 0.0, 1.0,
            0.0, 1.0, 0.0, 1.0,
        ];

        // upload vertices
        ctx.buffer_data_1(
            Ctx::ARRAY_BUFFER,
            Some(&stdweb::web::TypedArray::from(points.as_slice()).buffer()),
            Ctx::STATIC_DRAW,
        );

        // // texture coords
        // ctx.bind_buffer(Ctx::ARRAY_BUFFER, tex_buf.as_ref());

        // #[rustfmt::skip]
        // let texs = vec![
        //     0.0, 0.0,
        //     1.0, 0.0,
        //     0.0, 1.0,
        //     0.0, 0.0,
        //     1.0, 0.0,
        //     0.0, 1.0,
        // ];

        // // // upload texture coords
        // ctx.buffer_data_1(
        //     Ctx::ARRAY_BUFFER,
        //     Some(&stdweb::web::TypedArray::from(texs.as_slice()).buffer()),
        //     Ctx::STATIC_DRAW,
        // );

        console!(log, "Uploaded vertex data");

        ctx.use_program(Some(&prog));

        ctx.enable_vertex_attrib_array(a_pos);
        ctx.bind_buffer(Ctx::ARRAY_BUFFER, pos_buf.as_ref());
        ctx.vertex_attrib_pointer(a_pos, 4, Ctx::FLOAT, true, 0, 0);

        console!(log, "Bound position attributes");

        // ctx.enable_vertex_attrib_array(a_tex);
        // ctx.bind_buffer(Ctx::ARRAY_BUFFER, tex_buf.as_ref());
        // ctx.vertex_attrib_pointer(a_tex, 2, Ctx::FLOAT, false, 0, 0);

        // console!(log, "Bound texture attributes");

        let waker = Rc::new(Mutex::new(None));

        (
            Gfx {
                surface: Surf {
                    ctx: Rc::new(ctx),
                    waker: Rc::clone(&waker),
                },
            },
            waker,
        )
    }
}
