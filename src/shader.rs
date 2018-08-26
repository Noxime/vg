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
