mod broker;
mod stream;

use crate::{
    client::broker::BrokerConnection,
    error::Result,
    protocol::messages::metadata::{MetadataRequest, MetadataRequestTopic},
};
use futures::future::try_join_all;
use rand::{prelude::SliceRandom, thread_rng};

pub struct KafkaClient {
    pub brokers: Vec<BrokerConnection>,
}

pub struct ClientBuilder {
    pub brokers: Vec<String>,
    pub client_id: Option<String>,
    pub max_message_size: usize,
}

impl KafkaClient {
    pub fn new(brokers: Vec<String>) -> ClientBuilder {
        ClientBuilder {
            brokers,
            client_id: None,
            max_message_size: 100 * 1024 * 1024, // 100 MB
        }
    }

    async fn request_metadata(&self, topics: Vec<String>) -> Result<()> {
        let request = MetadataRequest {
            topics: topics
                .into_iter()
                .map(|topic| MetadataRequestTopic { name: topic })
                .collect(),
            allow_auto_topic_creation: None,
        };

        let mut brokers: Vec<&BrokerConnection> = self.brokers.iter().collect();
        brokers.shuffle(&mut thread_rng());

        for broker in brokers {
            Self::connect(broker).await?;
        }
        Ok(())
    }

    async fn connect(broker: &BrokerConnection) -> Result<()> {
        Ok(())
    }
}

impl ClientBuilder {
    pub async fn build(self) -> Result<KafkaClient> {
        Ok(KafkaClient {
            brokers: try_join_all(
                self.brokers
                    .into_iter()
                    .map(|broker| BrokerConnection::new(broker, self.client_id.clone())),
            )
            .await?,
        })
    }
}
