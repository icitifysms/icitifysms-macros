use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

/// Derive macro that generates:
///
/// impl MyEnum {
///     pub fn parse(s: &str) -> eyre::Result<Self> {
///         if let Ok(value) = Self::from_str(s) { ... } else { ... }
///     }
/// }
///
/// Requirements on the enum:
/// - `Self: std::str::FromStr` (e.g. via `#[derive(strum::EnumString)]`)
/// - `Self: enum_iterator::Sequence` (e.g. via `#[derive(enum_iterator::Sequence)]`)
/// - `Self: ToString` / Display (so `to_string()` works)
pub fn parse_enum_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Only allow enums
    if !matches!(input.data, Data::Enum(_)) {
        return quote! {
            compile_error!("EnumParseWithList can only be used on enums");
        }
        .into();
    }

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Parse from string; on error, list all possible variants.
            pub fn parse(s: &str) -> ::eyre::Result<Self> {
                use ::std::str::FromStr;

                if let ::std::result::Result::Ok(value) = <Self as FromStr>::from_str(s) {
                    ::std::result::Result::Ok(value)
                } else {
                    let variants_as_string = ::enum_iterator::all::<Self>()
                        .map(|v| ::std::string::ToString::to_string(&v))
                        .collect::<::std::vec::Vec<_>>()
                        .join(", ");

                    let error = ::std::format!(
                        "Invalid variant '{s}'. Possible values: {variants_as_string}"
                    );

                    ::std::result::Result::Err(::eyre::eyre!(error))
                }
            }
        }
    };

    expanded.into()
}
