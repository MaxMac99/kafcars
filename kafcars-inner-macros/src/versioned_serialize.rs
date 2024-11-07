use std::error::Error;
use darling::FromAttributes;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, Type};
use syn::spanned::Spanned;
use crate::common::{darling_to_syn, option_type, FieldOptions};

#[derive(FromAttributes)]
#[darling(attributes(kafka))]
pub struct VersionedSerializeOptions {
    #[darling(default)]
    pub max_version: i16,
    #[darling(default)]
    pub tag_version: Option<i16>,
}

pub fn derive_versioned_serialize_with_options(options: VersionedSerializeOptions, input: &DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
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
        impl <W: std::io::Write> crate::protocol::serializer::SerializeVersioned<W> for #ident #ty_generics #where_clause {
            fn serialize_versioned(&self, writer: &mut W, version: i16) -> Result<(), crate::protocol::error::SerializationError> {
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

pub fn derive_versioned_serialize(input: &DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let named_type_options =
        VersionedSerializeOptions::from_attributes(&input.attrs[..]).map_err(darling_to_syn)?;
    derive_versioned_serialize_with_options(named_type_options, input)
}

fn get_schema_def(s: &DataStruct, error_span: Span) -> Result<TokenStream, Vec<syn::Error>> {
    let mut field_exprs = vec![];
    match s.fields {
        Fields::Named(ref a) => {
            for field in a.named.iter() {
                let name = &field.ident;
                let field_attrs =
                    FieldOptions::from_attributes(&field.attrs[..]).map_err(darling_to_syn)?;
                let field_type = &field.ty;
                let call_serialize = if option_type(field_type).is_some() {
                    let call_serialize = if let Some(serialize) = field_attrs.serialize_with {
                        quote! {
                            #serialize(#name, writer)?;
                        }
                    } else {
                        quote! {
                            #name.serialize_versioned(writer, version)?;
                        }
                    };
                    quote! {
                        if let Some(#name) = &self.#name {
                            #call_serialize
                        }
                    }
                } else {
                    if let Some(serialize) = field_attrs.serialize_with {
                        quote! {
                            #serialize(#name, writer)?;
                        }
                    } else {
                        quote! {
                            self.#name.serialize_versioned(writer, version)?;
                        }
                    }
                };
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
                        if #condition {
                            #call_serialize
                        }
                    }
                } else {
                    call_serialize
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
        Ok(())
    })
}
