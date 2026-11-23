
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error};

pub fn update_releases_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return Error::new_spanned(
                name,
                "UpdateReleases can only be derived for enums",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut conversions = Vec::new();
    let mut reverse_conversions = Vec::new();
    let mut executions = Vec::new();
    let mut displays = Vec::new();

    for variant in variants {
        let variant_name = &variant.ident;

        if !matches!(variant.fields, syn::Fields::Unit) {
            return Error::new_spanned(
                variant,
                "UpdateReleases variants must be unit variants",
            )
            .to_compile_error()
            .into();
        }

        let variant_string = variant_name.to_string();

        let Some(version) = variant_string.strip_prefix("V") else {
            return Error::new_spanned(
                variant_name,
                "Release variants must start with V (e.g. V8_31_0)",
            )
            .to_compile_error()
            .into();
        };

        let parts: Vec<u64> = match version
            .split('_')
            .map(str::parse::<u64>)
            .collect::<Result<Vec<_>, std::num::ParseIntError>>()
        {
            Ok(parts) if parts.len() == 3 => parts,
            _ => {
                return Error::new_spanned(
                    variant_name,
                    "Invalid release version. Expected V8_31_0",
                )
                .to_compile_error()
                .into();
            }
        };

        let major = parts[0];
        let minor = parts[1];
        let patch = parts[2];

        let version_string = version.replace('_', ".");

        let module = syn::Ident::new(
            &format!("v{version}"),
            variant_name.span(),
        );

        conversions.push(quote! {
            (#major, #minor, #patch) => Ok(Self::#variant_name)
        });

        reverse_conversions.push(quote! {
            #name::#variant_name => {
                semver::Version::new(#major, #minor, #patch)
            }
        });

        displays.push(quote! {
            #name::#variant_name => #version_string
        });

        executions.push(quote! {
            Self::#variant_name => {
                #module::execute(pool).await
            }
        });
    }

    quote! {
        impl TryFrom<&semver::Version> for #name {
            type Error = eyre::Error;

            fn try_from(
                version: &semver::Version,
            ) -> eyre::Result<Self> {
                if !version.pre.is_empty() || !version.build.is_empty() {
                    eyre::bail!(
                        "Unsupported release: {version}"
                    );
                }

                match (
                    version.major,
                    version.minor,
                    version.patch,
                ) {
                    #(#conversions,)*
                    _ => eyre::bail!(
                        "Unsupported release: {version}"
                    ),
                }
            }
        }

        impl From<#name> for semver::Version {
            fn from(release: #name) -> Self {
                match release {
                    #(#reverse_conversions,)*
                }
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(
                &self,
                f: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                let version = match self {
                    #(#displays,)*
                };

                write!(f, "{version}")
            }
        }

        impl #name {
            pub async fn execute(
                &self,
                pool: &sqlx::MySqlPool,
            ) -> eyre::Result<()> {
                match self {
                    #(#executions,)*
                }
            }
        }
    }
    .into()
}
