mod openapi;
mod enum_extra;
mod cli_enum;
mod reverse_ord;
mod parse_enum;

use proc_macro::TokenStream;

use crate::{cli_enum::derive_cli_enum_impl, enum_extra::derive_enum_extra_impl, openapi::secured_path::secured_path_impl, parse_enum::parse_enum_impl, reverse_ord::derive_reverse_ord_impl};




#[proc_macro_attribute]
pub fn secured_path(args: TokenStream, input: TokenStream) -> TokenStream {
    secured_path_impl(args, input)
}



/// Derive:
/// - From<Enum> for sea_query::Value   (using Enum::to_string())
/// - PartialEq<str> for Enum           (case-insensitive)
/// - PartialEq<T: AsRef<str>> for Enum (case-insensitive)
///
/// REQUIREMENTS:
///   - The enum must implement `ToString` (usually via `Display` or `derive_more::Display`)
#[proc_macro_derive(EnumExtra)]
pub fn derive_enum_extra(input: TokenStream) -> TokenStream {
    derive_enum_extra_impl(input)
}





#[proc_macro_derive(CliEnum)]
pub fn derive_cli_enum(input: TokenStream) -> TokenStream {
    derive_cli_enum_impl(input)
}


#[proc_macro_derive(ReverseOrd)]
pub fn derive_reverse_ord(input: TokenStream) -> TokenStream {
    derive_reverse_ord_impl(input)
}



#[proc_macro_derive(ParseEnum)]
pub fn parse_enum(input: TokenStream) -> TokenStream {
    parse_enum_impl(input)
}