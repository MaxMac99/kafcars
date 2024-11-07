use std::error::Error;
use darling::FromAttributes;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, Type};
use syn::spanned::Spanned;
use crate::common::{darling_to_syn, option_type, FieldOptions};

#[derive(FromAttributes)]
#[darling(attributes(kafka))]
pub struct VersionedDeserializeOptions {
    #[darling(default)]
    pub max_version: i16,
    #[darling(default)]
    pub tag_version: Option<i16>,
}

pub fn derive_versioned_deserialize_with_options(options: VersionedDeserializeOptions, input: &DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let schema_def = match &input.data {
        Data::Struct(s) => get_schema_def(s, input.ident.span())?,
        _ => {
            return Err(vec![syn::Error::new(
                input.ident.span(),
                "Only structs are supported",
            )])
        }
    };

    let ident = &input.ident;
    let generics = &input.generics;
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let max_version = options.max_version;
    Ok(quote! {
        impl <R: std::io::Read> crate::protocol::deserializer::DeserializeVersioned<R> for #ident #ty_generics #where_clause {
            fn deserialize_versioned(data: &mut R, version: i16) -> Result<Self, crate::protocol::error::SerializationError> {
                if version > #max_version {
                    return Err(crate::protocol::error::SerializationError::UnsupportedVersion {
                        max_version: #max_version,
                        given_version: version,
                    });
                }
                #schema_def
            }
        }
    })
}

pub fn derive_versioned_deserialize(input: &DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let named_type_options =
        VersionedDeserializeOptions::from_attributes(&input.attrs[..]).map_err(darling_to_syn)?;
    derive_versioned_deserialize_with_options(named_type_options, input)
}

fn get_schema_def(s: &DataStruct, error_span: Span) -> Result<TokenStream, Vec<syn::Error>> {
    let mut field_exprs = vec![];
    let mut names = vec![];
    match s.fields {
        Fields::Named(ref a) => {
            for field in a.named.iter() {
                let name = &field.ident;
                let field_attrs =
                    FieldOptions::from_attributes(&field.attrs[..]).map_err(darling_to_syn)?;
                let mut field_type = &field.ty;
                if let Some(option_type) = option_type(field_type) {
                    field_type = option_type;
                }
                let condition = if let (Some(min_version), Some(max_version)) = (field_attrs.min_version, field_attrs.max_version) {
                    Some(quote! {
                        version >= #min_version && version <= #max_version
                    })
                } else if let Some(min_version) = field_attrs.min_version {
                    Some(quote! {
                        version >= #min_version
                    })
                } else if let Some(max_version) = field_attrs.max_version {
                    Some(quote! {
                        version <= #max_version
                    })
                } else {
                    None
                };
                field_exprs.push(if let Some(condition) = condition {
                    quote! {
                        let #name = if #condition {
                            Some(<#field_type as crate::protocol::deserializer::DeserializeVersioned<R>>::deserialize_versioned(data, version)?)
                        } else {
                            None
                        };
                    }
                } else {
                    quote! {
                        let #name = <#field_type as crate::protocol::deserializer::DeserializeVersioned<R>>::deserialize_versioned(data, version)?;
                    }
                });
                names.push(quote! {
                    #name
                });
            }
        },
        Fields::Unnamed(_) => {
            return Err(vec![syn::Error::new(
                error_span,
                "Tuple structs are not supported",
            )])
        },
        Fields::Unit => {
            return Err(vec![syn::Error::new(
                error_span,
                "Unit structs are not supported",
            )])
        },
    }

    Ok(quote! {
        #(#field_exprs)*
        Ok(Self {
            #(#names),*
        })
    })
}
