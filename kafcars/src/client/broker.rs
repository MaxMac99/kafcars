use crate::{
    error::Result,
    protocol::messages::metadata::{MetadataRequest, MetadataRequestTopic},
};
use tokio::net::TcpStream;

pub struct BrokerConnection {
    broker: String,
    client_id: Option<String>,
}

impl BrokerConnection {
    pub async fn new(broker: String, client_id: Option<String>) -> Result<Self> {
        let connection = TcpStream::connect(broker.clone()).await?;
        Ok(Self { broker, client_id })
    }

    pub async fn request_metadata(&self, topics: Vec<String>) -> Result<()> {
        Ok(())
    }
}

async fn metadata_request(request: &MetadataRequest, connection: &BrokerConnection) {}
