use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_enum_extra_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = input.ident;

    let expanded = quote! {
        // ---- From<Enum> for sea_query::Value ----
        impl ::core::convert::From<#enum_ident> for ::sea_query::Value {
            fn from(p: #enum_ident) -> Self {
                p.to_string().into()
            }
        }

        // ---- PartialEq<str> for Enum ----
        impl ::core::cmp::PartialEq<str> for #enum_ident {
            fn eq(&self, other: &str) -> bool {
                self.to_string().eq_ignore_ascii_case(other)
            }
            fn ne(&self, other: &str) -> bool {
                !self.eq(other)
            }
        }

        // ---- PartialEq<T: AsRef<str>> for Enum ----
        impl<T> ::core::cmp::PartialEq<T> for #enum_ident
        where
            T: ::core::convert::AsRef<str>,
        {
            fn eq(&self, other: &T) -> bool {
                self.to_string().eq_ignore_ascii_case(other.as_ref())
            }

            fn ne(&self, other: &T) -> bool {
                !self.eq(other)
            }
        }

        // ALSO let &str, String, etc compare on the left:
        impl ::core::cmp::PartialEq<#enum_ident> for str {
            fn eq(&self, other: &#enum_ident) -> bool {
                other.eq(&self)
            }
            fn ne(&self, other: &#enum_ident) -> bool {
                !self.eq(other)
            }
        }

        impl ::core::cmp::PartialEq<#enum_ident> for ::std::string::String {
            fn eq(&self, other: &#enum_ident) -> bool {
                other.eq(self)
            }
            fn ne(&self, other: &#enum_ident) -> bool {
                !self.eq(other)
            }
        }

        // ---- inherent all() using enum-iterator ----
        impl #enum_ident {
            /// Returns all variants of the enum in declaration order.
            /// Requires `#[derive(enum_iterator::Sequence)]` on the enum.
            pub fn all() -> ::std::vec::Vec<Self> {
                ::enum_iterator::all::<Self>().collect::<::std::vec::Vec<_>>()
            }
        }
    };

    TokenStream::from(expanded)
}
