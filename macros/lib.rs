use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[proc_macro]
pub fn return_with_try(input: TokenStream) -> TokenStream {
    let input = TokenStream2::from(input);

    quote! {
        match (|| Some({ #input }))() {
            Some(v) => v,
            None => return,
        }
    }.into()
}