use crate::protocol::error::SerializationError;
use serde::de;
use snafu::Snafu;
use std::{backtrace::Backtrace, fmt::Display};
use tokio::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(transparent)]
    Io {
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    Serialization { source: SerializationError },
    #[snafu(display("Error deserializing kafka message: {message}"))]
    Deserialize { message: String },
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        DeserializeSnafu {
            message: msg.to_string(),
        }
        .build()
    }
}
