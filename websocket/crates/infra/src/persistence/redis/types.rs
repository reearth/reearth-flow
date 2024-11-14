use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FlowEncodedUpdate {
    pub update: Vec<u8>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlowUpdate {
    pub stream_id: Option<String>,
    pub update: Vec<u8>,
    pub updated_by: Option<String>,
}

pub type StreamEntry = (String, Vec<u8>);
pub type StreamEntries = Vec<StreamEntry>;
pub type StreamItem = (String, StreamEntries);
pub type StreamItems = Vec<StreamItem>;
