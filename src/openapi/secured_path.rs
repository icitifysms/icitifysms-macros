use proc_macro::TokenStream;
use quote::quote;

const BEARER: &str = "bearerAuth";
pub fn secured_path_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_ts = proc_macro2::TokenStream::from(args);
    let input_ts = proc_macro2::TokenStream::from(input);

    let out = quote! {
        #[utoipa::path(
            #args_ts,
            security((#BEARER = []))
        )]
        #input_ts
    };
    out.into()
}