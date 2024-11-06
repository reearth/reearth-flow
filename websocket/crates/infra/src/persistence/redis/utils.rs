use super::errors::FlowProjectRedisDataManagerError;

pub fn encode_state_data(data: Vec<u8>) -> Result<String, FlowProjectRedisDataManagerError> {
    Ok(serde_json::to_string(&data)?)
}

pub fn decode_state_data(data_string: String) -> Result<Vec<u8>, FlowProjectRedisDataManagerError> {
    Ok(serde_json::from_str(&data_string)?)
}
