use crate::protocol::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MetadataRequest {
    pub topics: Vec<MetadataRequestTopic>,

    pub allow_auto_topic_creation: Option<bool>,
}

#[derive(Debug)]
pub struct MetadataRequestTopic {
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MetadataResponse {
    /// The duration in milliseconds for which the request was throttled due to
    /// a quota violation, or zero if the request did not violate any quota.
    ///
    /// Added in version 3
    pub throttle_time_ms: Option<i32>,

    /// Each broker in the response
    pub brokers: Vec<MetadataResponseBroker>,

    /// The cluster ID that responding broker belongs to.
    ///
    /// Added in version 2
    pub cluster_id: Option<String>,

    /// The ID of the controller broker.
    ///
    /// Added in version 1
    pub controller_id: Option<i32>,

    /// Each topic in the response
    pub topics: Vec<MetadataResponseTopic>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MetadataResponseBroker {
    /// The broker ID
    pub node_id: i32,
    /// The broker hostname
    pub host: String,
    /// The broker port
    pub port: i32,
    /// Added in version 1
    pub rack: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MetadataResponseTopic {
    /// The topic error if any
    pub error: Option<Error>,
    /// The topic name
    pub name: String,
    /// True if the topic is internal
    pub is_internal: Option<bool>,
    /// Each partition in the topic
    pub partitions: Vec<MetadataResponsePartition>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MetadataResponsePartition {
    /// The partition error if any
    pub error: Option<Error>,
    /// The partition index
    pub partition_index: i32,
    /// The ID of the leader broker
    pub leader_id: i32,
    /// The set of all nodes that host this partition
    pub replica_nodes: Vec<i32>,
    /// The set of all nodes that are in sync with the leader for this partition
    pub isr_nodes: Vec<i32>,
}
