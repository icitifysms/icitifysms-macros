use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data};


pub fn update_releases_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => panic!("UpdateReleases can only be derived for enums"),
    };

    let mut conversions = Vec::new();
    let mut executions = Vec::new();

    for variant in variants {
        let variant_name = &variant.ident;

        let version = variant_name
            .to_string()
            .strip_prefix("V")
            .unwrap()
            .replace('_', ".");

        let parts: Vec<_> = version
            .split('.')
            .map(|part| part.parse::<u64>().unwrap())
            .collect();

        let major = parts[0];
        let minor = parts[1];
        let patch = parts[2];

        let module = syn::Ident::new(
            &format!("v{}", version.replace('.', "_")),
            variant_name.span(),
        );

        conversions.push(quote! {
            (#major, #minor, #patch) => Ok(Self::#variant_name)
        });

        executions.push(quote! {
            Self::#variant_name => {
                crate::updates::releases::#module::execute(update_id, db).await
            }
        });
    }

    quote! {
        impl TryFrom<&semver::Version> for #name {
            type Error = eyre::Error;

            fn try_from(
                version: &semver::Version
            ) -> eyre::Result<Self> {
                match (
                    version.major,
                    version.minor,
                    version.patch
                ) {
                    #(#conversions,)*
                    _ => eyre::bail!(
                        "Unsupported release: {version}"
                    ),
                }
            }
        }

        impl #name {
            pub async fn execute(
                &self,
                update_id: &str,
                db: &sqlx::MySqlPool,
            ) -> eyre::Result<()> {
                match self {
                    #(#executions,)*
                }
            }
        }
    }
    .into()
}