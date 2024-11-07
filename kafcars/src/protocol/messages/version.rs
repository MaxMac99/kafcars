use kafcars_inner_macros::KafkaRequest;
use crate::protocol::{api_key::ApiKey, messages::KafkaRequest};
use crate::protocol::messages::TaggedFields;
use crate::protocol::serializer::serialize_compact_string;

#[derive(Debug, KafkaRequest)]
#[kafka(response = "ApiVersionsResponse", api_key = "ApiKey::ApiVersions", max_version = "4")]
pub struct ApiVersionsRequest {
    #[kafka(min_version = "3", serialize_with = "serialize_compact_string")]
    pub client_software_name: Option<String>,
    #[kafka(min_version = "3", serialize_with = "serialize_compact_string")]
    pub client_software_version: Option<String>,
    #[kafka(min_version = "3")]
    pub tagged_fields: Option<TaggedFields>,
}

pub struct ApiVersionsResponse {}
