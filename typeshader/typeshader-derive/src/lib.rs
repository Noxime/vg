extern crate proc_macro;
extern crate proc_macro2;
use proc_macro::TokenStream;
use proc_macro2::Span;

use std::collections::HashMap;
use std::path::PathBuf;

use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parse_macro_input, punctuated::Punctuated, Ident, LitStr, Token, Visibility};

mod codegen;
use codegen::signature::ProgramSignature;

#[derive(Debug)]
struct Error {
    span: Option<Span>,
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    Codegen(codegen::Error),
    File(String),
    StageRedef,
    SourceRedef,
    SpirvLit,
    SpirvLength,
}

impl From<codegen::Error> for Error {
    fn from(e: codegen::Error) -> Error {
        Error {
            span: None,
            kind: ErrorKind::Codegen(e),
        }
    }
}

mod kw {
    use syn::custom_keyword;
    // shader stages
    custom_keyword!(vertex);
    custom_keyword!(fragment);
    // shader languages
    custom_keyword!(glsl);
    custom_keyword!(hlsl);
    custom_keyword!(msl);
    custom_keyword!(spirv);
    // input method
    custom_keyword!(file);
}

#[derive(Debug)]
struct Programs(Punctuated<Program, Token![;]>);

impl Parse for Programs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Programs(input.parse_terminated(Program::parse)?))
    }
}

#[derive(Debug)]
struct Program {
    vis: Visibility,
    ident: Ident,
    stages: Punctuated<Stage, Token![,]>,
}

impl Parse for Program {
    fn parse(input: ParseStream) -> Result<Self> {
        let vis = input.parse()?;
        let ident = input.parse()?;
        input.parse::<Token![=]>()?;

        let content;
        braced!(content in input);

        Ok(Program {
            vis,
            ident,
            stages: content.parse_terminated(Stage::parse)?,
        })
    }
}

#[derive(Debug)]
struct Stage {
    type_: StageType,
    ident: Ident,
    sources: Punctuated<Source, Token![,]>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum StageType {
    Vertex,
    Fragment,
}

impl Parse for Stage {
    fn parse(input: ParseStream) -> Result<Self> {
        let peek = input.lookahead1();

        // check shader stage type
        let type_ = if peek.peek(kw::vertex) {
            StageType::Vertex
        } else if peek.peek(kw::fragment) {
            StageType::Fragment
        } else {
            return Err(peek.error());
        };

        // dismiss type as it was peeked
        let ident = input.parse()?;

        let content;
        braced!(content in input);

        Ok(Stage {
            type_,
            ident,
            sources: content.parse_terminated(Source::parse)?,
        })
    }
}

#[derive(Debug)]
struct Source {
    lang: Lang,
    ident: Ident,
    value: Value,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Lang {
    Glsl,
    Hlsl,
    Msl,
    Spirv,
}

impl Parse for Source {
    fn parse(input: ParseStream) -> Result<Self> {
        let peek = input.lookahead1();

        // parse source language
        let lang = if peek.peek(kw::glsl) {
            Lang::Glsl
        } else if peek.peek(kw::hlsl) {
            Lang::Hlsl
        } else if peek.peek(kw::msl) {
            Lang::Msl
        } else if peek.peek(kw::spirv) {
            Lang::Spirv
        } else {
            return Err(peek.error());
        };

        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let value = input.parse()?;

        Ok(Source { lang, ident, value })
    }
}

#[derive(Debug)]
enum Value {
    Str(LitStr),
    File(LitStr),
}

impl Value {
    fn to_string(self) -> std::result::Result<String, Error> {
        match self {
            Value::Str(s) => Ok(s.value()),
            Value::File(path) => {
                // try to load the source file
                let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
                let pathbuf = PathBuf::from(root).join(path.value());

                match std::fs::read_to_string(pathbuf) {
                    Ok(c) => Ok(c),
                    Err(e) => Err(Error {
                        span: Some(path.span()),
                        kind: ErrorKind::File(format!("{}", e)),
                    }),
                }
            }
        }
    }

    fn to_vec(self) -> std::result::Result<Vec<u32>, Error> {
        match self {
            Value::Str(s) => Err(Error {
                span: Some(s.span()),
                kind: ErrorKind::SpirvLit,
            }),
            Value::File(path) => {
                // try to load the source file
                let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
                let pathbuf = PathBuf::from(root).join(path.value());

                match std::fs::read(pathbuf) {
                    Ok(v) => vec8to32(path, v),
                    Err(e) => Err(Error {
                        span: Some(path.span()),
                        kind: ErrorKind::File(format!("{}", e)),
                    }),
                }
            }
        }
    }
}

impl Parse for Value {
    fn parse(input: ParseStream) -> Result<Self> {
        let peek = input.lookahead1();
        if peek.peek(kw::file) {
            let _ = input.parse::<kw::file>()?;
            // file path, text or bytes
            Ok(Value::File(input.parse()?))
        } else if peek.peek(LitStr) {
            // string literal, raw sourcecode
            Ok(Value::Str(input.parse()?))
        } else {
            Err(peek.error())
        }
    }
}

#[proc_macro]
pub fn shader(input: TokenStream) -> TokenStream {
    let mut code = vec![];

    for prog in parse_macro_input!(input as Programs).0 {
        let Program { vis, ident, stages } = prog;

        let (sig, vertex, fragment) = match try_build(stages) {
            Ok(v) => v,
            Err(error) => {
                let span = error.span.unwrap_or(ident.span());
                let msg = match error.kind {
                    ErrorKind::Codegen(e) => format!("Codegen: {:?}", e),
                    ErrorKind::File(s) => s,
                    ErrorKind::StageRedef => "Shader stage defined multiple times".into(),
                    ErrorKind::SourceRedef => "Shader stage source defined multiple times".into(),
                    ErrorKind::SpirvLit => "SPIR-V cannot be passed as a literal; use `spirv: file \"shader.spv\"`".into(),
                    ErrorKind::SpirvLength => "SPIR-V file length must be a multiple of 4. Make sure your file is not corrupted".into(),
                };

                return syn::Error::new(span, msg).to_compile_error().into();
            }
        };

        let vg = vertex.glsl;
        let vh = vertex.hlsl;
        let vm = vertex.msl;
        let vs = vertex.spirv;
        let fg = fragment.glsl;
        let fh = fragment.hlsl;
        let fm = fragment.msl;
        let fs = fragment.spirv;

        code.push(quote_spanned! {ident.span()=>
            #vis const #ident: typeshader::Program<#sig> = unsafe {
                typeshader::Program::new(
                    typeshader::Shader::new(#vg, #vh, #vm, &[#(#vs),*]),
                    typeshader::Shader::new(#fg, #fh, #fm, &[#(#fs),*]),
                )
            };
        })
    }

    let res = quote! {
        // TYPESHADER
        #(#code)*
    };

    res.into()
}

fn try_build(
    stages: Punctuated<Stage, Token![,]>,
) -> std::result::Result<(ProgramSignature, codegen::Sources, codegen::Sources), Error> {
    // codegen handles all shader analysis and transpilation
    let mut codegen = codegen::Codegen::new()?;

    // go through all stages and collect them up, erroring if redefined
    let mut chosen = HashMap::new();
    for stage in stages {
        if chosen.contains_key(&stage.type_) {
            return Err(Error {
                span: Some(stage.ident.span()),
                kind: ErrorKind::StageRedef,
            });
        } else {
            chosen.insert(stage.type_, stage.sources);
        }
    }

    for (stage, sources) in chosen.into_iter() {
        let mut glsl = None;
        let mut hlsl = None;
        let mut msl = None;
        let mut spirv = None;
        for source in sources {
            if match source.lang {
                Lang::Glsl => glsl.replace(source.value.to_string()?).is_some(),
                Lang::Hlsl => hlsl.replace(source.value.to_string()?).is_some(),
                Lang::Msl => msl.replace(source.value.to_string()?).is_some(),
                Lang::Spirv => spirv.replace(source.value.to_vec()?).is_some(),
            } {
                return Err(Error {
                    span: Some(source.ident.span()),
                    kind: ErrorKind::SourceRedef,
                });
            }
        }
        // compile the shader stage and translate any error messages
        codegen.add(stage, glsl, hlsl, msl, spirv)?;
    }
    codegen.finish().map_err(From::from)
}

fn vec8to32(path: LitStr, v: Vec<u8>) -> std::result::Result<Vec<u32>, Error> {
    if v.len() % 4 != 0 {
        return Err(Error {
            span: Some(path.span()),
            kind: ErrorKind::SpirvLength,
        });
    }

    let mut res = vec![];
    res.reserve(v.len() / 4); // pre-allocate
    for i in 0..v.len() / 4 {
        // TODO: Check this is correct... Idk if spirv defines itself to be little endian
        res.push(u32::from_le_bytes([
            v[i * 4 + 3],
            v[i * 4 + 2],
            v[i * 4 + 1],
            v[i * 4 + 0],
        ]));
    }
    Ok(res)
}
