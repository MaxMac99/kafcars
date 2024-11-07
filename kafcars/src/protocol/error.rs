use snafu::Snafu;
use std::{io, io::ErrorKind};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {}

#[derive(Snafu, Debug)]
pub enum SerializationError {
    #[snafu(transparent)]
    Io { source: io::Error },
    #[snafu(display("Unknown value for type"))]
    UnknownValue,
    #[snafu(display("Value is too large"))]
    Overflow,
    #[snafu(display("Message is malformed: {message}"))]
    Malformed { message: String },
    #[snafu(display("The given version is not supported ({given_version} > {max_version})"))]
    UnsupportedVersion {
        max_version: i16,
        given_version: i16,
    },
}

impl From<SerializationError> for io::Error {
    fn from(value: SerializationError) -> Self {
        if let SerializationError::Io { source, .. } = value {
            source
        } else {
            io::Error::new(ErrorKind::InvalidData, value)
        }
    }
}
