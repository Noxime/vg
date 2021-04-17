use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[proc_macro_attribute]
pub fn game(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input: TokenStream2 = input.into();

    let expanded = quote! {
        fn main() {
            #input

            ::vg::__vg_start(main);
        }
    };

    expanded.into()
}
