use anyhow::*;
use glob::glob;
use std::fs::{read_to_string, write};
use std::path::PathBuf;

struct ShaderData {
    src: String,
    src_path: PathBuf,
    spv_path: PathBuf,
    kind: shaderc::ShaderKind,
}

impl ShaderData {
    pub fn load(src_path: PathBuf) -> Result<Self> {
        let extension = src_path
            .extension()
            .context("File has no extension")?
            .to_str()
            .context("Extension cannot be converted to &str")?;
        let kind = match extension {
            "vs" => shaderc::ShaderKind::Vertex,
            "fs" => shaderc::ShaderKind::Fragment,
            "cs" => shaderc::ShaderKind::Compute,
            _ => bail!("Unsupported shader: {}", src_path.display()),
        };

        let src = read_to_string(src_path.clone())?;
        let spv_path = PathBuf::from(std::env::var("OUT_DIR")?)
            .join("out")
            .with_file_name(src_path.file_name().unwrap())
            .with_extension(format!("{}.spv", extension));

        Ok(Self {
            src,
            src_path,
            spv_path,
            kind,
        })
    }
}

fn main() -> Result<()> {
    // Collect all shaders recursively within /src/
    let mut shader_paths = [
        glob("./src/**/*.vs")?,
        glob("./src/**/*.fs")?,
        glob("./src/**/*.cs")?,
    ];

    // This could be parallelized
    let shaders = shader_paths
        .iter_mut()
        .flatten()
        .map(|glob_result| ShaderData::load(glob_result?))
        .collect::<Vec<Result<_>>>()
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    let mut compiler = shaderc::Compiler::new().context("Unable to create shader compiler")?;
    let mut opts = shaderc::CompileOptions::new().unwrap();
    opts.set_include_callback(|name, _, inc, _| {
        let path = PathBuf::from(inc).with_file_name(name);
        if !path.exists() {
            panic!("Can't find {}", path.display());
        }
        Ok(shaderc::ResolvedInclude {
            resolved_name: format!("{:?}", path),
            content: read_to_string(path).unwrap(),
        })
    });

    // This can't be parallelized. The [shaderc::Compiler] is not
    // thread safe. Also, it creates a lot of resources. You could
    // spawn multiple processes to handle this, but it would probably
    // be better just to only compile shaders that have been changed
    // recently.
    for shader in shaders {
        // This tells cargo to rerun this script if something in /src/ changes.
        println!("cargo:rerun-if-changed={:?}", shader.src_path);

        let compiled = compiler.compile_into_spirv(
            &shader.src,
            shader.kind,
            &shader.src_path.to_str().unwrap(),
            "main",
            Some(&opts),
        )?;
        write(shader.spv_path, compiled.as_binary_u8())?;
    }

    Ok(())
}
