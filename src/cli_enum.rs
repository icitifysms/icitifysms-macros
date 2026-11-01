use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

/// #[derive(CliEnum)] entry point
pub fn derive_cli_enum_impl(input: TokenStream) -> TokenStream {
    // Parse the input as a DeriveInput
    let input = parse_macro_input!(input as DeriveInput);

    // Destructure to get what we need
    let DeriveInput { ident: enum_name, attrs, data, .. } = input;

    // Ensure it's an enum
    let Data::Enum(DataEnum { variants, .. }) = data else {
        return syn::Error::new_spanned(
            enum_name,
            "#[derive(CliEnum)] can only be applied to enums",
        )
        .to_compile_error()
        .into();
    };

    // Generate Cli<EnumName>
    let cli_enum_name = format_ident!("Cli{}", enum_name);

    // Extract variant identifiers only (ignore tuple fields)
    let variant_idents: Vec<_> =
        variants.iter().map(|v| v.ident.clone()).collect();

    // Copy only serde attributes from the original enum
    // e.g. #[serde(rename_all = "kebab-case")]
    let serde_attrs: Vec<_> = attrs
        .into_iter()
        .filter(|attr| attr.path().is_ident("serde"))
        .collect();

    // Build match arms for conversion
    let match_arms = variant_idents.iter().map(|variant| {
        quote! {
            #cli_enum_name::#variant => constructor(stringify!(#variant))
        }
    });

    let expanded = quote! {
        #[derive(
            clap::ValueEnum,
            utoipa::ToSchema,
            serde::Serialize,
            serde::Deserialize,
            Clone,
            Copy,
            Debug
        )]
        #(#serde_attrs)*
        pub enum #cli_enum_name {
            #(#variant_idents),*
        }

        impl #cli_enum_name {
            pub fn to_enum<F>(self, constructor: F) -> #enum_name
            where
                F: FnOnce(&str) -> #enum_name
            {
                match self {
                    #(#match_arms),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
