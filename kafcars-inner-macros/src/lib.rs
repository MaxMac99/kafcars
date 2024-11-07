mod versioned_deserialize;
mod common;
mod kafka_request;
mod versioned_serialize;

use crate::common::to_compile_errors;
use crate::kafka_request::derive_kafka_request;
use crate::versioned_deserialize::derive_versioned_deserialize;
use darling::FromAttributes;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput};
use crate::versioned_serialize::derive_versioned_serialize;

#[proc_macro_derive(VersionedDeserialize, attributes(kafka))]
pub fn proc_macro_derive_versioned_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    derive_versioned_deserialize(&mut input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

#[proc_macro_derive(VersionedSerialize, attributes(kafka))]
pub fn proc_macro_derive_versioned_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    derive_versioned_serialize(&mut input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

#[proc_macro_derive(KafkaRequest, attributes(kafka))]
pub fn proc_macro_derive_kafka_request(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    derive_kafka_request(&mut input)
        .unwrap_or_else(to_compile_errors)
        .into()
}
