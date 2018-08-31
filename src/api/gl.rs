// the OpenGL (es2.0) implementation of the graphics API

extern crate gl;
extern crate glutin;

use api::*;

use self::gl::types::*;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

pub struct GLApi {
    size: (usize, usize)
}

impl GfxApi for GLApi {
    fn clear(&self, r: f32, g: f32, b: f32) {
        unsafe {
            gl::ClearColor(r, g, b, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // match gl::GetError() {
            //     0 => (),
            //     v => log!("GL error: {}", v),
            // }
        }
    }

    fn resize(&mut self, width: usize, height: usize) {
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            self.size = (width, height);
        }
    }

    fn size(&self) -> (usize, usize) {
        self.size
    }

    fn compile_shader(&self, vertex: &str, fragment: &str) -> Result<Shader, ShaderError> {
        log!("GL: Compiling new shader program");
        log!("GL: Compiling vertex shader");
        let vs = self.compile_one(vertex, gl::VERTEX_SHADER)?;
        log!("GL: Vertex shader done");
        log!("GL: Compiling fragment shader");
        let fs = self.compile_one(fragment, gl::FRAGMENT_SHADER)?;
        log!("GL: Fragment shader done");

        Ok(Shader::new(unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);
            log!("GL: Shader linked");

            // Get the link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
            log!("GL: Shader link status: {}", status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::with_capacity(len as usize);
                log!("GL: Error log length: {}", len);
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );

                gl::DeleteProgram(program);

                return Err(ShaderError::CompileError(
                    String::from_utf8(buf).map_err(|_| ShaderError::StringError)?,
                ));
            }

            log!("GL: Shader created with internal id: {}", program);
            program
        }))
    }

    fn upload_texture(&self, width: usize, height: usize, data: Vec<u8>, smooth: bool) -> Texture {
        let mut tex = 0;
        unsafe {
            gl::GenTextures(1, &mut tex);
            log!("GL: new texture with internal id {}", tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);

            // smooth or closest filtering?
            if smooth {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
            } else {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
            }

            // upload the texture data
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as _,
                width as _,
                height as _,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );

            log!("GL: new texture uploaded to GPU");
        }

        Texture(tex)
    }

    fn debug_draw_vertices(&self, shader: &Shader, vertices: &Vec<((f32, f32, f32), (f32, f32))>, texture: Option<&Texture>) {
        unsafe {
            gl::UseProgram(shader.0);

            // if we use a texture, bind it here
            if let Some(Texture(tex)) = texture {
                let uni = gl::GetUniformLocation(shader.0, b"u_texture\0" as *const _ as *const _) as u32;
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, *tex);
                gl::Uniform1i(uni as _, 0);
            }

            // update some generic uniforms
            {
                let uni = gl::GetUniformLocation(shader.0, b"u_resolution\0" as *const _ as *const _) as u32;
                gl::Uniform2f(uni as _, self.size.0 as _, self.size.1 as _);
            }

            // great... on some platforms c_char is u8 and on some i8, this repeated *const _ will convert it (shrug)
            let pos =
                gl::GetAttribLocation(shader.0, b"a_position\0" as *const _ as *const _) as u32;
            let coord =
                gl::GetAttribLocation(shader.0, b"a_texcoord\0" as *const _ as *const _) as u32;
            // log!("attrib loc: {}", pos);

            gl::EnableVertexAttribArray(pos);
            gl::EnableVertexAttribArray(coord);

            let mut verts = Vec::new();
            let mut coords = Vec::new();

            for v in vertices {
                verts.push(v.0);
                coords.push(v.1);
            }

            use std::mem::size_of;
            // point to the position data
            gl::VertexAttribPointer(
                pos,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<(f32, f32, f32)>() as i32,
                verts.as_ptr() as *const c_void,
            );
            // texcoord data
            gl::VertexAttribPointer(
                coord,
                2,
                gl::FLOAT,
                gl::FALSE,
                size_of::<(f32, f32)>() as i32,
                coords.as_ptr() as *const c_void,
            );

            gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as i32);

            gl::DisableVertexAttribArray(pos);
            gl::DisableVertexAttribArray(coord);
        }
    }
}

impl GLApi {
    pub fn new(width: usize, height: usize) -> Self {
        unsafe {
            // enable debugging TODO: Figure out why this doesnt actually call
            gl::DebugMessageCallback(Self::debug_callback, 0 as *const c_void);

            // enable transparency
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        GLApi {
            size: (width, height)
        }
    }

    extern "system" fn debug_callback(
        _: GLenum,
        _: GLenum,
        _: GLuint,
        _: GLenum,
        _: GLsizei,
        _: *const GLchar,
        _: *mut c_void,
    ) {
        //TODO: Use this
        log!("GL: Callback");
    }

    fn compile_one(&self, source: &str, type_: GLenum) -> Result<GLuint, ShaderError> {
        Ok(unsafe {
            let shader = gl::CreateShader(type_);
            gl::ShaderSource(
                shader,
                1,
                &CString::new(source)
                    .map_err(|_| ShaderError::StringError)?
                    .as_ptr(),
                ptr::null(),
            );
            gl::CompileShader(shader);

            // Get the compile status
            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
            log!("GL: Shader compile status: {}", status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::with_capacity(len as usize);
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );

                // don't leak
                gl::DeleteShader(shader);

                return Err(ShaderError::CompileError(
                    String::from_utf8(buf).map_err(|_| ShaderError::StringError)?,
                ));
            }
            shader
        })
    }
}
