use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn derive_reverse_ord_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let data_enum = match input.data {
        Data::Enum(e) => e,
        _ => {
            return syn::Error::new_spanned(
                name,
                "ReverseOrd can only be derived for enums",
            )
            .to_compile_error()
            .into();
        }
    };

    // Ensure all variants are fieldless (unit)
    for v in &data_enum.variants {
        if !matches!(v.fields, Fields::Unit) {
            return syn::Error::new_spanned(
                &v.ident,
                "ReverseOrd currently only supports fieldless (unit) enums",
            )
            .to_compile_error()
            .into();
        }
    }

    // Generate rank match arms in declaration order
    let rank_arms = data_enum
        .variants
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            let v_ident = &v.ident;
            quote! {
                Self::#v_ident => #idx
            }
        });

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #[inline]
            fn __reverse_ord_rank(&self) -> usize {
                match self {
                    #( #rank_arms, )*
                }
            }
        }

        // Structural equality via discriminant for fieldless enums
        impl #impl_generics ::core::cmp::PartialEq for #name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                ::core::mem::discriminant(self) == ::core::mem::discriminant(other)
            }
        }

        impl #impl_generics ::core::cmp::Eq for #name #ty_generics #where_clause {}

        impl #impl_generics ::core::cmp::PartialOrd for #name #ty_generics #where_clause {
            fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
                // reversed order: first-declared = greatest
                Some(other.__reverse_ord_rank().cmp(&self.__reverse_ord_rank()))
            }
        }

        impl #impl_generics ::core::cmp::Ord for #name #ty_generics #where_clause {
            fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
                // reversed order: first-declared = greatest
                other.__reverse_ord_rank().cmp(&self.__reverse_ord_rank())
            }
        }
    };

    TokenStream::from(expanded)
}
