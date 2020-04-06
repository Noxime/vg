use std::collections::{HashMap, BTreeMap};
use std::fmt::{Display, Formatter, Result as FmtRes};

use syn::{punctuated::Punctuated, Token};
use quote::quote;

use crate::codegen::Error;
mod sigtype;
pub(crate) use sigtype::Type;

#[derive(Debug)]
pub(crate) struct ProgramSignature {
    uniforms: BTreeMap<Binding, Type>,
    inputs: BTreeMap<Location, Type>,
    outputs: BTreeMap<Location, Type>,
}

impl ProgramSignature {
    pub(crate) fn from(vertex: ShaderSignature, fragment: ShaderSignature) -> Self {
        let mut uniforms = BTreeMap::new();
        uniforms.extend(vertex.uniforms);
        uniforms.extend(fragment.uniforms);
        let uniforms = uniforms.into_iter().map(|(k, (_, t))| (k, t)).collect();

        let inputs = vertex.inputs.into_iter().map(|(k, (_, t))| (k, t)).collect();
        let outputs = fragment.outputs.into_iter().map(|(k, (_, t))| (k, t)).collect();
        ProgramSignature {
            uniforms,
            inputs,
            outputs,
        }
    }
}

impl quote::ToTokens for ProgramSignature {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {

        let mut uni = quote!{};
        let mut ins = quote!{};
        let mut outs = quote!{};

        let mut last_b = Binding(0);
        for (b, t) in self.uniforms.iter() {
            // fill unpopulated bindings with skip types
            while last_b != *b {
                uni.extend(quote! {
                    typeshader::S,
                });
                last_b = Binding(last_b.0 + 1);
            }
            uni.extend(quote! { #t, });
        }

        let mut last_l = Location(0);
        for (l, t) in self.inputs.iter() {
            // fill unpopulated locations with skip types
            while last_l != *l {
                ins.extend(quote! {
                    typeshader::S,
                });
                last_l = Location(last_l.0 + 1);
            }
            ins.extend(quote! { #t, });
        }

        let mut last_l = Location(0);
        for (l, t) in self.outputs.iter() {
            // fill unpopulated locations with skip types
            while last_l != *l {
                outs.extend(quote! {
                    typeshader::S,
                });
                last_l = Location(last_l.0 + 1);
            }
            outs.extend(quote! { #t, });
        }

        tokens.extend(quote! {
            (#uni), (#ins), (#outs)
        });
    }
}

// uniform binding index
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Binding(pub u32);

// attribute location
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Location(pub u32);

// type signature for a shader
#[derive(Debug, Eq)]
pub(crate) struct ShaderSignature {
    pub(crate) uniforms: HashMap<Binding, (String, Type)>,
    pub(crate) inputs: HashMap<Location, (String, Type)>,
    pub(crate) outputs: HashMap<Location, (String, Type)>,
}

impl ShaderSignature {
    // hlsl prefixes the names with its own stuff, so remove those
    pub(crate) fn fix_hlsl(mut self) -> Self {
        for (n, _) in self.inputs.values_mut() {
            if n.starts_with("stage_input_") {
                *n = n.split_off(12);
            }
        }

        self.outputs
            .retain(|_, (n, _)| n != "_entryPointOutput_gl_Position");
        for (n, _) in self.outputs.values_mut() {
            if n.starts_with("_entryPointOutput_") {
                *n = n.split_off(18);
            }
        }

        self
    }
}

impl PartialEq for ShaderSignature {
    fn eq(&self, other: &Self) -> bool {
        self.inputs == other.inputs && self.outputs == other.outputs
    }
}

impl Display for ShaderSignature {
    fn fmt(&self, f: &mut Formatter) -> FmtRes {
        write!(f, "{:#?}", self)
    }
}
