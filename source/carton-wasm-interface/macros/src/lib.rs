extern crate proc_macro;

use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::ItemFn;
use syn::parse::Parse;
use syn::parse_macro_input;

#[proc_macro]
pub fn infer(input: TokenStream) -> TokenStream {
    let input: ItemFn = parse_macro_input!(input);
    let fn_name = &input.sig.ident;

    let expanded = quote! {
        #input

        #[no_mangle]
        pub extern "C" fn _init_infer() {
            carton_wasm_interface::set_infer_fn(#fn_name);
        }
    };

    TokenStream::from(expanded)
}