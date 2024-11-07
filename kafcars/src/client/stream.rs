use crate::{
    client::stream::ResponseError::{InvalidHeader, UnknownRequest},
    protocol::{
        deserializer::{DeserializeVersioned, DeserializeVersionedInto, VersionedDeserializer},
        error::SerializationError,
        messages::{header::ResponseHeader, TaggedFields},
    },
};
use futures::{StreamExt, TryStream, TryStreamExt};
use log::warn;
use serde::Deserialize;
use snafu::{OptionExt, ResultExt, Snafu};
use std::{
    collections::HashMap,
    io::{Cursor, Read},
    ops::DerefMut,
    sync::Arc,
};
use std::cmp::min;
use std::sync::atomic::{AtomicI32, Ordering};
use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    spawn,
    sync::{oneshot::Sender, Mutex},
};
use tokio::task::JoinHandle;
use tokio_serde::{formats::Json, Deserializer, Framed};
use tokio_util::{
    bytes::BytesMut,
    codec::{FramedRead, LengthDelimitedCodec},
};
use crate::protocol::api_key::ApiKey;
use crate::protocol::messages::{ApiVersion, ApiVersionRange, KafkaRequest, KafkaResponse};
use crate::protocol::messages::header::RequestHeader;
use crate::protocol::messages::version::ApiVersionsRequest;

#[derive(Debug)]
pub struct ConnectionStream {
    stream_write: Arc<Mutex<OwnedWriteHalf>>,
    state: Arc<Mutex<HashMap<i32, ActiveRequest>>>,
    response_handler: JoinHandle<crate::error::Result<()>>,
    correlation_id: AtomicI32,
}

#[derive(Debug)]
pub struct Response {
    header: ResponseHeader,
    payload: Cursor<Vec<u8>>,
}

#[derive(Debug, Snafu)]
pub enum RequestError {
    #[snafu(display("Could not read message"))]
    ReadError { source: SerializationError },
    #[snafu(display("Could not find a version match for api key {api_key}"))]
    NoVersionMatch { api_key: ApiKey },
}

#[derive(Debug, Snafu)]
pub enum ResponseError {
    #[snafu(display("Cannot read message header, ignoring message"))]
    InvalidHeader { source: SerializationError },
    #[snafu(display("Got a response for an unknown request {correlation_id}"))]
    UnknownRequest { correlation_id: i32 },
}

#[derive(Debug)]
struct ActiveRequest {
    channel: Sender<Result<Response, RequestError>>,
    use_tagged_fields_in_response: bool,
}

impl DeserializeVersionedInto<Cursor<Vec<u8>>> for Cursor<Vec<u8>> {
    fn deserialize_versioned_into(
        data: Cursor<Vec<u8>>,
        _: i16,
    ) -> Result<Self, SerializationError> {
        Ok(data)
    }
}

impl ConnectionStream {
    pub fn new(stream: TcpStream) -> Self {
        let (stream_read, stream_write) = stream.into_split();
        let state = Arc::new(Mutex::new(HashMap::default()));

        let join = spawn(Self::stream_reader_task(stream_read, state.clone()));

        Self {
            stream_write: Arc::new(Mutex::new(stream_write)),
            state,
            response_handler: join,
            correlation_id: AtomicI32::new(0),
        }
    }

    async fn stream_reader_task(
        stream_read: OwnedReadHalf,
        state: Arc<Mutex<HashMap<i32, ActiveRequest>>>,
    ) -> crate::error::Result<()> {
        let stream = FramedRead::new(stream_read, LengthDelimitedCodec::new());
        let deserializer = VersionedDeserializer::new(0);
        let mut stream: Framed<
            FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
            Cursor<Vec<u8>>,
            (),
            VersionedDeserializer,
        > = Framed::new(stream, deserializer);
        stream
            .try_for_each_concurrent(1, |val| {
                let state = state.clone();
                async move {
                    if let Err(err) = Self::read_with_raw_response_message(val, state).await {
                        warn!("Could not read message: {}", err);
                    }
                    Ok(())
                }
            })
            .await?;
        Ok(())
    }

    async fn read_with_raw_response_message(
        mut data: Cursor<Vec<u8>>,
        state: Arc<Mutex<HashMap<i32, ActiveRequest>>>,
    ) -> Result<(), ResponseError> {
        let mut header =
            ResponseHeader::deserialize_versioned(&mut data, 0).context(InvalidHeaderSnafu)?;
        let active_request = state
            .lock()
            .await
            .deref_mut()
            .remove(&header.correlation_id)
            .context(UnknownRequestSnafu {
                correlation_id: header.correlation_id,
            })?;
        if active_request.use_tagged_fields_in_response {
            header.tagged_fields = match TaggedFields::deserialize_versioned(&mut data, 0) {
                Ok(fields) => Some(fields),
                Err(e) => {
                    active_request
                        .channel
                        .send(Err(RequestError::ReadError { source: e }))
                        .ok();
                    return Ok(());
                }
            }
        }

        active_request
            .channel
            .send(Ok(Response {
                header,
                payload: data,
            }))
            .ok();

        Ok(())
    }
    
    pub async fn sync_versions(&mut self) -> crate::error::Result<()> {
        'upper_bound: for upper_bound in (ApiVersionsRequest::API_VERSION_RANGE.min..=ApiVersionsRequest::API_VERSION_RANGE.max).rev() {
            let version_ranges = HashMap::from([(
                ApiKey::ApiVersions,
                ApiVersionRange {
                    min: ApiVersionsRequest::API_VERSION_RANGE.min,
                    max: upper_bound,
                },
            )]);
            
            let body = ApiVersionsRequest {
                client_software_name: Some(String::from(env!("CARGO_PKG_NAME"))),
                client_software_version: Some(String::from(env!("CARGO_PKG_VERSION"))),
                tagged_fields: Some(TaggedFields::default()),
            };
            
            'send: loop {
                
            }
        }
        Ok(())
    }
    
    async fn send_request_with_version_ranges<R>(
        &self,
        message: R,
        version_ranges: HashMap<ApiKey, ApiVersionRange>,
    ) -> Result<(), RequestError>
    where
        R: KafkaRequest,
        R::KafkaResponse: KafkaResponse,
    {
        let body_version = version_ranges
            .get(&R::API_KEY)
            .and_then(|range_server| match_versions(*range_server, R::API_VERSION_RANGE))
            .ok_or(RequestError::NoVersionMatch {
                api_key: R::API_KEY,
            })?;
        
        let use_tagged_fields_in_request = R::TAGGED_FIELDS_MIN_VERSION
            .map(|v| body_version >= v)
            .unwrap_or(false);
        let use_tagged_fields_in_response = R::KafkaResponse::TAGGED_FIELDS_MIN_VERSION
            .map(|v| body_version >= v)
            .unwrap_or(false);
        
        let correlation_id = self.correlation_id.fetch_add(1, Ordering::SeqCst);
        
        let header = RequestHeader {
            correlat
        }
        
        Ok(())
    }
}

fn match_versions(range0: ApiVersionRange, range1: ApiVersionRange) -> Option<ApiVersion> {
    if range0.min <= range1.max && range1.min <= range0.max {
        Some(min(range0.max, range1.max))
    } else {
        None
    }
}


