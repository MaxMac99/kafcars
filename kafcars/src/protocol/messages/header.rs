use crate::protocol::messages::{ApiVersion, TaggedFields};
use kafcars_inner_macros::{VersionedDeserialize, VersionedSerialize};
use crate::protocol::api_key::ApiKey;

#[derive(Debug, VersionedSerialize)]
#[kafka(max_version = 2)]
pub struct RequestHeader {
    pub request_api_version: ApiVersion,
    pub request_api_key: ApiKey,
    pub correlation_id: i32,
    #[kafka(min_version = 1, serialize_with = "serialize_nullable_string")]
    pub client_id: String,
}

#[derive(Debug, VersionedDeserialize)]
#[kafka(max_version = 1)]
pub struct ResponseHeader {
    pub correlation_id: i32,
    #[kafka(min_version = 1)]
    pub tagged_fields: Option<TaggedFields>,
}
