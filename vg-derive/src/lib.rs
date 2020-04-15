use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;

use std::{
    fs::File,
    io::{copy, Cursor},
    path::Path,
};

#[derive(Debug, FromMeta)]
struct MacroArgs {
    assets: Option<String>,
}

#[proc_macro_attribute]
pub fn game(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let name = func.sig.ident.clone();

    let attrs = syn::parse_macro_input!(attr as syn::AttributeArgs);
    let args = match MacroArgs::from_list(&attrs) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let mut zip = zip::ZipWriter::new(Cursor::new(vec![]));

    if let Some(assets) = args.assets {
        // TODO: Asset tiering

        // package all files into zip
        for entry in walkdir::WalkDir::new(&assets) {
            let entry = entry.unwrap();
            let path = entry.path();
            let name = path.strip_prefix(Path::new(&assets)).unwrap();

            if path.is_file() {
                zip.start_file_from_path(name, Default::default()).unwrap();
                // read file into zip
                copy(&mut File::open(path).unwrap(), &mut zip).unwrap();
            } else if name.as_os_str().len() != 0 {
                // add dir
                zip.add_directory_from_path(name, Default::default())
                    .unwrap();
            }
        }

    // zip.add_directory_from_path(std::path::Path::new(&path), Default::default()).expect("Failed to add asset dir");
    } else {
        panic!("no assets")
    }

    zip.set_comment(format!("VG/{} Asset bundle", env!("CARGO_PKG_VERSION")));

    let assets = zip
        .finish()
        .expect("Failed to finish asset bundle ZIP")
        .into_inner();

    (quote! {
        fn main() {
            #func
            vg::__startup(#name, &[#(#assets),*])
        }
    })
    .into()
}
