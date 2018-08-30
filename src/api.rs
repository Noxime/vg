extern crate gl;
extern crate glutin;

use self::gl::types::*;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

#[derive(Debug)]
pub struct Shader(u32);
impl Shader {
    pub fn new(id: u32) -> Self {
        Shader(id)
    }
}

#[derive(Debug)]
pub enum ShaderError {
    StringError,
    CompileError(String),
}

pub trait GfxApi {
    // clear the current buffer with specified color
    fn clear(&self, r: f32, g: f32, b: f32);
    // take shader sources and compile them down to a shader
    fn compile_shader(&self, vertex: &str, fragment: &str) -> Result<Shader, ShaderError>;
    // basic vertex drawing method, do not use for production code as it is slow
    fn debug_draw_vertices(&self, shader: &Shader, vertices: &Vec<(f32, f32, f32)>);
}

pub struct GLApi;
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

    fn compile_shader(&self, vertex: &str, fragment: &str) -> Result<Shader, ShaderError> {
        log!("Compiling new shader program");
        log!("Compiling vertex shader");
        let vs = self.compile_one(vertex, gl::VERTEX_SHADER)?;
        log!("Vertex shader done");
        log!("Compiling fragment shader");
        let fs = self.compile_one(fragment, gl::FRAGMENT_SHADER)?;
        log!("Fragment shader done");

        Ok(Shader::new(unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);
            log!("Shader linked");

            // Get the link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
            log!("Shader link status: {}", status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::with_capacity(len as usize);
                log!("Error log length: {}", len);
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

            log!("Shader created with internal id: {}", program);
            program
        }))
    }

    fn debug_draw_vertices(&self, shader: &Shader, vertices: &Vec<(f32, f32, f32)>) {
        unsafe {
            gl::UseProgram(shader.0);
            // great... on some platforms c_char is u8 and on some i8, this repeated *const _ will convert it (shrug)
            let pos =
                gl::GetAttribLocation(shader.0, b"a_position\0" as *const _ as *const _) as u32;
            // log!("attrib loc: {}", pos);

            gl::EnableVertexAttribArray(pos);

            use std::mem::size_of;
            gl::VertexAttribPointer(
                pos,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<(f32, f32, f32)>() as i32,
                vertices.as_ptr() as *const c_void,
            );
            gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as i32);

            gl::DisableVertexAttribArray(pos);
        }
    }
}

impl GLApi {
    pub fn new() -> Self {
        unsafe {
            gl::DebugMessageCallback(Self::debug_callback, 0 as *const c_void);
        }
        GLApi
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
            log!("Shader compile status: {}", status);

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
