use crate::protocol::error::{
    SerializationError,
    SerializationError::{Malformed, UnknownValue},
};
use std::{
    io::{Cursor, Read},
    pin::Pin,
};
use tokio_util::bytes::BytesMut;
use crate::protocol::messages::ApiVersion;

pub trait DeserializeVersioned<R>
where
    R: Read,
{
    fn deserialize_versioned(data: &mut R, version: ApiVersion) -> Result<Self, SerializationError>
    where
        Self: Sized;
}

pub trait DeserializeVersionedInto<R>
where
    R: Read,
{
    fn deserialize_versioned_into(data: R, version: ApiVersion) -> Result<Self, SerializationError>
    where
        Self: Sized;
}

pub struct VersionedDeserializer {
    api_version: ApiVersion,
}

impl VersionedDeserializer {
    pub fn new(api_version: ApiVersion) -> Self {
        VersionedDeserializer { api_version }
    }
}

impl<T: DeserializeVersionedInto<Cursor<Vec<u8>>>> tokio_serde::Deserializer<T>
    for VersionedDeserializer
{
    type Error = SerializationError;

    fn deserialize(self: Pin<&mut Self>, src: &BytesMut) -> Result<T, Self::Error> {
        let cursor = Cursor::new(src.to_vec());
        T::deserialize_versioned_into(cursor, self.api_version)
    }
}

impl<R: Read> DeserializeVersioned<R> for bool {
    fn deserialize_versioned(data: &mut R, _: ApiVersion) -> Result<Self, SerializationError> {
        let mut buf = [0u8; 1];
        data.read_exact(&mut buf)?;
        Ok(match buf[0] {
            0 => false,
            1 => true,
            _ => Err(UnknownValue)?,
        })
    }
}

impl<R: Read> DeserializeVersioned<R> for i8 {
    fn deserialize_versioned(data: &mut R, _: ApiVersion) -> Result<Self, SerializationError> {
        let mut buf = [0u8; 1];
        data.read_exact(&mut buf)?;
        Ok(i8::from_be_bytes(buf))
    }
}

impl<R: Read> DeserializeVersioned<R> for i16 {
    fn deserialize_versioned(data: &mut R, _: ApiVersion) -> Result<Self, SerializationError> {
        let mut buf = [0u8; 2];
        data.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }
}

impl<R: Read> DeserializeVersioned<R> for i32 {
    fn deserialize_versioned(data: &mut R, _: ApiVersion) -> Result<Self, SerializationError> {
        let mut buf = [0u8; 4];
        data.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }
}

impl<R: Read> DeserializeVersioned<R> for u32 {
    fn deserialize_versioned(data: &mut R, _: ApiVersion) -> Result<Self, SerializationError> {
        let mut buf = [0u8; 4];
        data.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }
}

impl<R: Read> DeserializeVersioned<R> for i64 {
    fn deserialize_versioned(data: &mut R, _: ApiVersion) -> Result<Self, SerializationError> {
        let mut buf = [0u8; 8];
        data.read_exact(&mut buf)?;
        Ok(i64::from_be_bytes(buf))
    }
}

pub fn deserialize_unsigned_var_int<R: Read>(data: &mut R) -> Result<u64, SerializationError> {
    let mut buf = [0u8; 1];
    let mut res: u64 = 0;
    let mut shift = 0;
    loop {
        data.read_exact(&mut buf)?;
        let c: u64 = buf[0].into();

        res |= (c & 0x7f) << shift;
        shift += 7;

        // First bit in byte indicates if finished
        if (c & 0x80) == 0 {
            break;
        }
        // Can only process 64 bits
        if shift > 63 {
            return Err(Malformed {
                message: "Overflow while reading unsigned var int".to_string(),
            });
        }
    }
    Ok(res)
}
