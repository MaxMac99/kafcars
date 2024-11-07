use crate::protocol::{
    api_key::ApiKey,
    deserializer::{deserialize_unsigned_var_int, DeserializeVersioned, DeserializeVersionedInto},
    error::SerializationError,
    messages::header::ResponseHeader,
};
use futures::StreamExt;
use std::{
    collections::HashMap,
    io::{Cursor, Read},
};
use std::io::Write;
use crate::protocol::messages::version::ApiVersionsRequest;
use crate::protocol::serializer::{serialize_unsigned_var_int, SerializeVersioned};

pub mod header;
pub mod metadata;
pub mod version;

pub type ApiVersion = i16;

pub trait KafkaRequest {
    type KafkaResponse;
    const API_KEY: ApiKey;
    const API_VERSION_RANGE: ApiVersionRange;
    const TAGGED_FIELDS_MIN_VERSION: Option<ApiVersion>;
}

pub trait KafkaResponse {
    const TAGGED_FIELDS_MIN_VERSION: Option<ApiVersion>;
}

#[derive(Debug, Copy, Clone)]
pub struct ApiVersionRange {
    pub min: ApiVersion,
    pub max: ApiVersion,
}

pub type TaggedFields = HashMap<u64, Vec<u8>>;

impl<W: Write> SerializeVersioned<W> for TaggedFields {
    fn serialize_versioned(&self, writer: &mut W, _: i16) -> Result<(), SerializationError>
    where
        Self: Sized
    {
        serialize_unsigned_var_int(self.len() as u64, writer)?;
        for (tag, values) in self {
            serialize_unsigned_var_int(*tag, writer)?;
            serialize_unsigned_var_int(values.len() as u64, writer)?;
            writer.write_all(values)?;
        }

        Ok(())
    }
}

impl<R: Read> DeserializeVersioned<R> for TaggedFields {
    fn deserialize_versioned(data: &mut R, _: i16) -> Result<Self, SerializationError> {
        let num_fields = deserialize_unsigned_var_int(data)?;
        if num_fields == 0 {
            return Ok(HashMap::new());
        }

        let mut fields = HashMap::new();
        let mut prev_tag = None;
        for _ in 0..=num_fields {
            let tag = deserialize_unsigned_var_int(data)?;
            if prev_tag.map(|prev_tag| tag <= prev_tag).unwrap_or(false) {
                return Err(SerializationError::Malformed {
                    message: format!("Invalid or out of order tag {}", tag),
                });
            }
            prev_tag = Some(tag);

            let size = deserialize_unsigned_var_int(data)?;
            let mut content = vec![0u8; size as usize];
            let mut chunk = data.take(size);
            chunk.read_to_end(&mut content)?;
            if fields.insert(tag, content).is_some() {
                return Err(SerializationError::Malformed {
                    message: format!("Tag {} already exists", tag),
                });
            }
        }

        Ok(fields)
    }
}
