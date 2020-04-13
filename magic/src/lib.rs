extern crate proc_macro;
extern crate cargo_vg;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

use quote::ToTokens;

use std::process::Command;
use std::path::Path;
use std::{fs, env};

// create a temp directory and CD into it, run and restore to original state
fn within(dir: impl AsRef<Path>, f: impl Fn()) {
    let dir = dir.as_ref();
    fs::remove_dir_all(dir);
    fs::create_dir_all(dir).expect("Failed to create directory");
    
    let old = env::current_dir().expect("Failed to get current dir");
    env::set_current_dir(dir).expect("Failed to set current dir");

    f();

    env::set_current_dir(old).expect("Failed to reset current dir");
    // fs::remove_dir_all(dir).expect("Failed to remove directory");
}

#[proc_macro_attribute]
pub fn vg(_attr: TokenStream, item: TokenStream) -> TokenStream {

    // let release = Path::new(".vg-release").exists();
    // fs::remove_file(".vg-release").unwrap();
    let release = false;

    within("__tmp", || {
        let _ = Command::new("cp")
            .arg("-r")
            .arg("..")
            .arg(".").output().expect("Copy failed");

        cargo_vg::build(release)
    });
    
    // let url = "https://owo.codes/noxim/vg/-/archive/current/vg-current.tar.gz?path=target-desktop";

    let mut input = syn::parse_macro_input!(item as syn::ItemFn);

    // change to normal fn main() {} block
    input.sig.asyncness = None;
    input.sig.generics = Default::default();
    input.sig.inputs = Default::default();
    input.block.stmts = vec![];

    input.into_token_stream().into()
}