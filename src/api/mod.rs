// generic internal api to access various graphics API's (like opengl)

mod gl;
pub use self::gl::GLApi;

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

#[derive(Debug)]
pub struct Texture(u32);

pub trait GfxApi {
    // clear the current buffer with specified color
    fn clear(&self, r: f32, g: f32, b: f32);
    // resize the current window
    fn resize(&self, width: usize, height: usize);
    // take shader sources and compile them down to a shader
    fn compile_shader(&self, vertex: &str, fragment: &str) -> Result<Shader, ShaderError>;
    // send some texture data (RGBA @ 8bits/channel) to the GPU and get a "handle" back to it
    fn upload_texture(&self, width: usize, height: usize, data: Vec<u8>, smooth: bool) -> Texture;
    // basic vertex drawing method, do not use for production code as it is slow
    fn debug_draw_vertices(&self, shader: &Shader, vertices: &Vec<((f32, f32, f32), (f32, f32))>, texture: Option<&Texture>);
}
