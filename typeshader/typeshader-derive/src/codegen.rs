use std::collections::HashMap;

use crate::{StageType, Lang};

use spirv_cross::{
    glsl, hlsl, msl,
    spirv::{Ast, Compile, Module, Parse, Target, Decoration},
};
use shaderc::{CompileOptions, ShaderKind, SourceLanguage};

pub(crate) mod signature;
use signature::{ProgramSignature, ShaderSignature, Type, Binding, Location};

// basically Option::unwrap_or_else, but lets you use ?
macro_rules! uore {
    ($e: expr, $ee: expr) => {
        match $e {
            Some(e) => e,
            None => $ee,
        }
    };
}

#[derive(Debug)]
pub(crate) enum Error {
    Init,
    NoStage(StageType),
    NoSource,
    Unsupported(String),
    Incompatible(Lang, ShaderSignature, ShaderSignature),
    IncompatibleStages(HashMap<Location, (String, Type)>, HashMap<Location, (String, Type)>),
    Cross(spirv_cross::ErrorCode),
    ShaderC(shaderc::Error),
}

impl From<spirv_cross::ErrorCode> for Error {
    fn from(e: spirv_cross::ErrorCode) -> Error {
        Error::Cross(e)
    }
}

impl From<shaderc::Error> for Error {
    fn from(e: shaderc::Error) -> Error {
        Error::ShaderC(e)
    }
}

// TODO: More descriptive name
pub(crate) struct Codegen {
    shaderc: shaderc::Compiler,
    vertex: Option<(Sources, ShaderSignature)>,
    fragment: Option<(Sources, ShaderSignature)>,
}

pub(crate) struct Sources {
    pub(crate) glsl: String,
    pub(crate) hlsl: String,
    pub(crate) msl: String,
    pub(crate) spirv: Vec<u32>,
}

impl Codegen {
    pub(crate) fn new() -> Result<Codegen, Error> {
        Ok(Codegen {
            shaderc: shaderc::Compiler::new().ok_or(Error::Init)?,
            vertex: None,
            fragment: None,
        })
    }

    // add some new source code to the on-going program
    pub(crate) fn add(
        &mut self,
        stage: StageType,
        glsl: Option<String>,
        hlsl: Option<String>,
        msl: Option<String>,
        spirv: Option<Vec<u32>>,
    ) -> Result<(), Error> {
        // unwrap or generate spir-v
        let spirv = if let Some(s) = spirv {
            s
        } else {
            if let Some(ref glsl) = glsl {
                self.to_spirv(stage, SourceLanguage::GLSL, &glsl)?
            } else if let Some(ref hlsl) = hlsl {
                self.to_spirv(stage, SourceLanguage::HLSL, &hlsl)?
            } else {
                // no MSL fallback here, since shaderc can't make spirv out of that
                return Err(Error::NoSource);
            }
        };

        // unwrap or generate
        let glsl = uore!(glsl, self.to_sl::<glsl::Target, Ast<glsl::Target>>(&spirv)?);
        let hlsl = uore!(hlsl, self.to_sl::<hlsl::Target, Ast<hlsl::Target>>(&spirv)?);
        let msl = uore!(msl, self.to_sl::<msl::Target, Ast<msl::Target>>(&spirv)?);

        // the type signature to compare against

        // compile HL back to spirv, note: MSL can't currently be compiled to spirv, so no verification
        let glsl_spirv = self.to_spirv(stage, SourceLanguage::GLSL, &glsl)?;
        let hlsl_spirv = self.to_spirv(stage, SourceLanguage::HLSL, &hlsl)?;

        // produce signatures out of all the sources
        let sig_spirv = self.signature::<glsl::Target>(&spirv)?;
        let sig_glsl = self.signature::<glsl::Target>(&glsl_spirv)?;
        let sig_hlsl = self.signature::<hlsl::Target>(&hlsl_spirv)?.fix_hlsl(); // guh...
        let sig_msl = self.signature::<msl::Target>(&spirv)?;

        if sig_spirv != sig_glsl {
            return Err(Error::Incompatible(Lang::Glsl, sig_spirv, sig_glsl))
        }

        if sig_spirv != sig_hlsl {
            return Err(Error::Incompatible(Lang::Hlsl, sig_spirv, sig_hlsl))
        }

        if sig_spirv != sig_msl {
            return Err(Error::Incompatible(Lang::Msl, sig_spirv, sig_msl))
        }

        let sources = Sources {
            glsl,
            hlsl,
            msl,
            spirv,
        };

        match stage {
            StageType::Vertex => self.vertex = Some((sources, sig_spirv)),
            StageType::Fragment => self.fragment = Some((sources, sig_spirv)),
        }

        Ok(())
    }

    pub(crate) fn finish(self) -> Result<(ProgramSignature, Sources, Sources), Error> {
        let (vertex_source, vertex_sig) = self.vertex.ok_or(Error::NoStage(StageType::Vertex))?;
        let (fragment_source, fragment_sig) = self.fragment.ok_or(Error::NoStage(StageType::Fragment))?;

        // make sure all the varyings are passed properly
        if vertex_sig.outputs != fragment_sig.inputs {
            return Err(Error::IncompatibleStages(vertex_sig.outputs, fragment_sig.inputs))
        }

        Ok((
            ProgramSignature::from(vertex_sig, fragment_sig),
            vertex_source,
            fragment_source,
        ))
    }

    // analyze spirv and produce a type signature
    fn signature<T>(&mut self, spirv: &Vec<u32>) -> Result<ShaderSignature, Error>
    where
        T: Target,
        Ast<T>: Parse<T> + Compile<T>,
    {
        let module = Module::from_words(spirv);
        let ast = Ast::<T>::parse(&module)?;
        let res = ast.get_shader_resources()?;

        let mut uniforms = HashMap::new();
        for res in res.uniform_buffers {
            uniforms.insert(Binding(ast.get_decoration(res.id, Decoration::Binding)?), (res.name, Type::try_from(&ast, ast.get_type(res.type_id)?)?));
        }

        let mut inputs = HashMap::new();
        for res in res.stage_inputs {
            inputs.insert(Location(ast.get_decoration(res.id, Decoration::Location)?), (res.name, Type::try_from(&ast, ast.get_type(res.type_id)?)?));
        }

        let mut outputs = HashMap::new();
        for res in res.stage_outputs {
            outputs.insert(Location(ast.get_decoration(res.id, Decoration::Location)?), (res.name, Type::try_from(&ast, ast.get_type(res.type_id)?)?));
        }

        Ok(ShaderSignature {
            uniforms,
            inputs,
            outputs,
        })
    }

    // generate languages out of spirv
    fn to_sl<T, A>(&mut self, spirv: &Vec<u32>) -> Result<String, Error>
    where
        A: Parse<T> + Compile<T>,
        T: Target,
    {
        let module = Module::from_words(spirv);
        let mut ast = A::parse(&module)?;
        Ok(ast.compile()?)
    }

    // generate spirv out of a high level language
    fn to_spirv(
        &mut self,
        stage: StageType,
        lang: SourceLanguage,
        sl: &String,
    ) -> Result<Vec<u32>, Error> {
        // convert data type
        let stage = match stage {
            StageType::Vertex => ShaderKind::Vertex,
            StageType::Fragment => ShaderKind::Fragment,
        };

        let mut opts = CompileOptions::new().expect("ShaderC options init failed");
        opts.set_source_language(lang);
        Ok(self
            .shaderc
            .compile_into_spirv(sl, stage, "shader", "main", Some(&opts))?
            .as_binary()
            .to_vec())
    }
}
