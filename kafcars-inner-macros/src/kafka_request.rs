use quote::quote;
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use syn::{Data, DeriveInput, Fields, Type};
use crate::common::{darling_to_syn, FieldOptions};
use crate::versioned_serialize::{derive_versioned_serialize_with_options, VersionedSerializeOptions};

#[derive(FromAttributes)]
#[darling(attributes(kafka))]
struct KafkaRequestOptions {
    /// Default: 0
    #[darling(default)]
    min_version: Option<i16>,
    #[darling(default)]
    max_version: i16,
    #[darling(default)]
    tag_version: Option<i16>,
    #[darling(default)]
    api_key: Option<Type>,
    #[darling(default)]
    response: Option<Type>,
}

pub fn derive_kafka_request(input: &DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let named_type_options =
        KafkaRequestOptions::from_attributes(&input.attrs[..]).map_err(darling_to_syn)?;
    let derive_serialize_opts = VersionedSerializeOptions {
        max_version: named_type_options.max_version,
        tag_version: named_type_options.tag_version,
    };
    let derive_serialize = derive_versioned_serialize_with_options(derive_serialize_opts, input)?;

    let ident = &input.ident;
    let generics = &input.generics;
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let response = named_type_options.response
        .ok_or(vec![syn::Error::new(
            input.ident.span(),
            "The response is required",
        )])?;
    let api_key = named_type_options.api_key
        .ok_or(vec![syn::Error::new(
            input.ident.span(),
            "The api key is required",
        )])?;
    let min_version = named_type_options.min_version.unwrap_or(0);
    let max_version = named_type_options.max_version;
    let tagged_field = find_tagged_field_version(&input);
    let tagged_field_version = tagged_field
        .map(|f| f.0)
        .map(|v| quote! {Some(#v)})
        .unwrap_or(quote! {None});
    Ok(quote! {
        #derive_serialize
        impl crate::protocol::messages::KafkaRequest for #ident #ty_generics #where_clause {
            type KafkaResponse = #response;
            const API_KEY: crate::protocol::api_key::ApiKey = #api_key;
            const API_VERSION_RANGE: crate::protocol::messages::ApiVersionRange = crate::protocol::messages::ApiVersionRange {
                min: #min_version, 
                max: #max_version,
            };
            const TAGGED_FIELDS_MIN_VERSION: Option<crate::protocol::messages::ApiVersion> = #tagged_field_version;
        }
    })
}

fn find_tagged_field_version(input: &DeriveInput) -> Option<(i16, &Ident)> {
    match &input.data {
        Data::Struct(s) => {
            match s.fields {
                Fields::Named(ref a) => {
                    for field in a.named.iter() {
                        let Ok(field_attrs) =
                            FieldOptions::from_attributes(&field.attrs[..]) else { continue; };
                        let Some(min_version) = field_attrs.min_version else { continue; };
                        let Some(name) = &field.ident else { continue; };
                        let Type::Path(path) = &field.ty else { continue; };
                        if path.path.segments.last().unwrap().ident.to_string() == "TaggedFields" {
                            return Some((min_version, name));
                        }
                    }
                    None
                }
                _ => None,
            }
        },
        _ => None,
    }
}
