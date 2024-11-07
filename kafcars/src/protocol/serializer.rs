use crate::protocol::error::{
    SerializationError,
    SerializationError::{Malformed, UnknownValue},
};
use std::{
    io::{Cursor, Read},
    pin::Pin,
};
use std::io::Write;
use tokio_util::bytes::BytesMut;

pub trait SerializeVersioned<W>
where
    W: Write,
{
    fn serialize_versioned(&self, writer: &mut W, version: i16) -> Result<(), SerializationError>
    where
        Self: Sized;
}

pub struct VersionedSerializer {
    api_version: i16,
}

impl VersionedSerializer {
    pub fn new(api_version: i16) -> Self {
        VersionedSerializer { api_version }
    }
}

impl<W: Write> SerializeVersioned<W> for bool {
    fn serialize_versioned(&self, writer: &mut W, _: i16) -> Result<(), SerializationError> {
        Ok(writer.write_all(&[if *self { 1 } else { 0 }])?)
    }
}

impl<W: Write> SerializeVersioned<W> for i8 {
    fn serialize_versioned(&self, writer: &mut W, _: i16) -> Result<(), SerializationError> {
        Ok(writer.write_all(&self.to_be_bytes())?)
    }
}

impl<W: Write> SerializeVersioned<W> for i16 {
    fn serialize_versioned(&self, writer: &mut W, _: i16) -> Result<(), SerializationError> {
        Ok(writer.write_all(&self.to_be_bytes())?)
    }
}

impl<W: Write> SerializeVersioned<W> for i32 {
    fn serialize_versioned(&self, writer: &mut W, _: i16) -> Result<(), SerializationError> {
        Ok(writer.write_all(&self.to_be_bytes())?)
    }
}

impl<W: Write> SerializeVersioned<W> for u32 {
    fn serialize_versioned(&self, writer: &mut W, _: i16) -> Result<(), SerializationError> {
        Ok(writer.write_all(&self.to_be_bytes())?)
    }
}

impl<W: Write> SerializeVersioned<W> for i64 {
    fn serialize_versioned(&self, writer: &mut W, _: i16) -> Result<(), SerializationError> {
        Ok(writer.write_all(&self.to_be_bytes())?)
    }
}

pub fn serialize_compact_string<W: Write>(val: &String, writer: &mut W) -> Result<(), SerializationError> {
    let len = u64::try_from(val.len() + 1).map_err(|_| SerializationError::Overflow)?;
    serialize_unsigned_var_int(len, writer)?;
    writer.write_all(val.as_bytes())?;
    Ok(())
}

pub fn serialize_unsigned_var_int<W: Write>(val: u64, writer: &mut W) -> Result<(), SerializationError> {
    let mut curr = val;
    loop {
        let mut c = u8::try_from(val & 0x7f)
            .expect("u64 to u8 with 0x7f mask should always work");
        curr >>= 7;
        if curr > 0 {
            c |= 0x80;
        }
        
        writer.write_all(&[c])?;
        if curr == 0 {
            break;
        }
    }
    Ok(())
}
