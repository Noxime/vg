use spirv_cross::spirv::{Ast, Compile, Parse, Target, Type as SpirvType};

use crate::codegen::Error;

use quote::ToTokens;

// basically Option::unwrap_or_else, but lets you use ?
macro_rules! uore {
    ($a: expr, $b: expr) => {
        match $a {
            Some(value) => value,
            None => $b,
        }
    };
}

// boilerplate for array handling
macro_rules! arr {
    ($array: expr, $ast: expr, $old: expr => $new: expr) => {
        uore!(
            $array.pop()
                .map(|n| Type::try_from($ast, $old).map(|t| Type::Array(Box::new(t), n as _)))
                .transpose()?,
            $new
        )
    };
}

// a shader type thats allowed in signature
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Type {
    Leaf(Leaf),
    Array(Box<Type>, usize),
    Struct(Vec<Type>),
    Vec2(Leaf),
    Vec3(Leaf),
    Vec4(Leaf),
    Mat2(Leaf),
    Mat3(Leaf),
    Mat4(Leaf),
}

use syn::{punctuated::Punctuated, Token};
use quote::quote;
impl quote::ToTokens for Type {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Type::Leaf(t) => quote! { #t },
            Type::Array(t, n) => quote! { [#t; #n] },
            Type::Struct(t) => quote! { typeshader::Struct<(#(#t,)*)> },
            Type::Vec2(t) => quote! { typeshader::Vector<#t, 2> },
            Type::Vec3(t) => quote! { typeshader::Vector<#t, 3> },
            Type::Vec4(t) => quote! { typeshader::Vector<#t, 4> },
            Type::Mat2(t) => quote! { typeshader::Matrix<#t, 2> },
            Type::Mat3(t) => quote! { typeshader::Matrix<#t, 3> },
            Type::Mat4(t) => quote! { typeshader::Matrix<#t, 4> },
        });
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Leaf {
    Boolean,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    AtomicCounter,
    F16,
    F32,
    F64,
    Image,
    SampledImage,
    Sampler,
}

impl quote::ToTokens for Leaf {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Leaf::Boolean => quote! { bool },
            Leaf::U8 => quote! { u8 },
            Leaf::I8 => quote! { i8 },
            Leaf::U16 => quote! { u16 },
            Leaf::I16 => quote! { i16 },
            Leaf::U32 => quote! { u32 },
            Leaf::I32 => quote! { i32 },
            Leaf::U64 => quote! { u64 },
            Leaf::I64 => quote! { i64 },
            Leaf::AtomicCounter => todo!(),
            Leaf::F16 => quote! { typeshader::f16 }, // TODO: blocked by lack of f16
            Leaf::F32 => quote! { f32 },
            Leaf::F64 => quote! { f64 },
            Leaf::Image => todo!(),
            Leaf::SampledImage => todo!(),
            Leaf::Sampler => todo!(),
        });
    }
}

fn vc2t(vecsize: u32, columns: u32, t: Leaf) -> Result<Type, Error> {
    let p = match t {
        Leaf::F16 => "lowp",
        Leaf::F32 => "mediump",
        Leaf::F64 => "highp",
        t => unreachable!("vec/mat of type `{:?}`", t),
    };
    Ok(match (vecsize, columns) {
        (1, 1) => Type::Leaf(Leaf::F32),
        (2, 1) => Type::Vec2(Leaf::F32),
        (3, 1) => Type::Vec3(Leaf::F32),
        (4, 1) => Type::Vec4(Leaf::F32),
        (2, 2) => Type::Mat2(Leaf::F32),
        (3, 3) => Type::Mat3(Leaf::F32),
        (4, 4) => Type::Mat4(Leaf::F32),
        (w, 1) => return Err(Error::Unsupported(format!("`{} vec{}`", p, w))),
        (w, h) => return Err(Error::Unsupported(format!("`{} mat{}x{}`", p, w, h))),
    })
}

impl Type {
    pub(crate) fn try_from<T>(ast: &Ast<T>, s: SpirvType) -> Result<Self, Error>
    where
        T: Target,
        Ast<T>: Parse<T> + Compile<T>,
    {
        Ok(match s {
            // struct handling
            SpirvType::Struct {
                member_types,
                mut array,
            } => {
                arr!(array, ast, SpirvType::Struct { array, member_types: member_types.clone() } =>
                    {
                        let mut fields = vec![];
                        for id in member_types {
                            fields.push(Type::try_from(ast, ast.get_type(id)?)?);
                        }
                        Type::Struct(fields)
                    }
                )
            }
            // vectors and matrices
            SpirvType::Half {
                vecsize,
                columns,
                mut array,
            } => arr!(array, ast, SpirvType::Float { array, vecsize, columns } =>
                vc2t(vecsize, columns, Leaf::F16)?
            ),
            SpirvType::Float {
                vecsize,
                columns,
                mut array,
            } => arr!(array, ast, SpirvType::Float { array, vecsize, columns } =>
                vc2t(vecsize, columns, Leaf::F32)?
            ),
            SpirvType::Double {
                vecsize,
                columns,
                mut array,
            } => arr!(array, ast, SpirvType::Float { array, vecsize, columns } =>
                vc2t(vecsize, columns, Leaf::F64)?
            ),

            // simple types
            SpirvType::Boolean { mut array } => {
                arr!(array, ast, SpirvType::Boolean { array } => Type::Leaf(Leaf::Boolean))
            }
            SpirvType::SByte { mut array } => {
                arr!(array, ast, SpirvType::SByte { array } => Type::Leaf(Leaf::I8))
            }
            SpirvType::UByte { mut array } => {
                arr!(array, ast, SpirvType::UByte { array } => Type::Leaf(Leaf::U8))
            }
            SpirvType::Short { mut array } => {
                arr!(array, ast, SpirvType::Short { array } => Type::Leaf(Leaf::I16))
            }
            SpirvType::UShort { mut array } => {
                arr!(array, ast, SpirvType::UShort { array } => Type::Leaf(Leaf::U16))
            }
            SpirvType::Int { mut array } => {
                arr!(array, ast, SpirvType::Int { array } => Type::Leaf(Leaf::I32))
            }
            SpirvType::UInt { mut array } => {
                arr!(array, ast, SpirvType::UInt { array } => Type::Leaf(Leaf::U32))
            }
            SpirvType::Int64 { mut array } => {
                arr!(array, ast, SpirvType::Int64 { array } => Type::Leaf(Leaf::I64))
            }
            SpirvType::UInt64 { mut array } => {
                arr!(array, ast, SpirvType::UInt64 { array } => Type::Leaf(Leaf::U64))
            }
            SpirvType::AtomicCounter { mut array } => {
                arr!(array, ast, SpirvType::AtomicCounter { array } => Type::Leaf(Leaf::AtomicCounter))
            }
            SpirvType::Image { mut array } => {
                arr!(array, ast, SpirvType::Image { array } => Type::Leaf(Leaf::Image))
            }
            SpirvType::SampledImage { mut array } => {
                arr!(array, ast, SpirvType::SampledImage { array } => Type::Leaf(Leaf::SampledImage))
            }
            SpirvType::Sampler { mut array } => {
                arr!(array, ast, SpirvType::Sampler { array } => Type::Leaf(Leaf::Sampler))
            }
            t => return Err(Error::Unsupported(format!("(internal error) type `{:?}` is not supported", t)))
        })
    }
}
