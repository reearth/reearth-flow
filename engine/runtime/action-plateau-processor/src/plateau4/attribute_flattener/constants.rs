use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) struct AttributePath {
    pub(super) attribute: String,
    pub(super) data_type: String,
    pub(super) json_path: String,
}

pub(super) static FLATTEN_ATTRIBUTES: Lazy<HashMap<String, Vec<AttributePath>>> = Lazy::new(|| {
	let data = include_str!("flatten_attributes.json");
    serde_json::from_str(data).unwrap()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_attributes() {
        let flatten_attributes = FLATTEN_ATTRIBUTES.clone();
        assert!(flatten_attributes.contains_key("bldg/bldg:Building"));
		assert!(flatten_attributes.contains_key("rwy/tran:AuxiliaryTrafficArea"));
		assert!(flatten_attributes.contains_key("rwy/tran:Railway"));
		assert!(flatten_attributes.contains_key("rwy/tran:TrafficArea"));
		assert!(flatten_attributes.contains_key("squr/tran:AuxiliaryTrafficArea"));
		assert!(flatten_attributes.contains_key("squr/tran:Square"));
		assert!(flatten_attributes.contains_key("squr/tran:TrafficArea"));
		assert!(flatten_attributes.contains_key("tran/tran:AuxiliaryTrafficArea"));
		assert!(flatten_attributes.contains_key("tran/tran:Road"));
		assert!(flatten_attributes.contains_key("tran/tran:TrafficArea"));
		assert!(flatten_attributes.contains_key("trk/tran:AuxiliaryTrafficArea"));
		assert!(flatten_attributes.contains_key("trk/tran:Track"));
		assert!(flatten_attributes.contains_key("trk/tran:TrafficArea"));
		assert!(flatten_attributes.contains_key("wwy/tran:TrafficArea"));
		assert!(flatten_attributes.contains_key("wwy/uro:Waterway"));
	}
}
