use std::collections::HashMap;

use crate::attribute::AttributeValue;
use crate::lod::LodMask;

pub type Metadata = HashMap<String, AttributeValue>;

pub const CITYGML_GML_ID: &str = "citygml_gml_id";
pub const CITYGML_FEATURE_TYPE: &str = "citygml_feature_type";
pub const CITYGML_LOD_MASK: &str = "citygml_lod_mask";

pub trait CitygmlFeatureExt {
    fn metadata(&self) -> &Metadata;

    fn citygml_gml_id(&self) -> Option<String> {
        self.metadata()
            .get(CITYGML_GML_ID)
            .and_then(|v| v.as_string())
    }

    fn citygml_feature_type(&self) -> Option<String> {
        self.metadata()
            .get(CITYGML_FEATURE_TYPE)
            .and_then(|v| v.as_string())
    }

    fn citygml_lod_mask(&self) -> Option<LodMask> {
        self.metadata().get(CITYGML_LOD_MASK).and_then(|v| {
            if let AttributeValue::Number(n) = v {
                n.as_u64().map(|v| LodMask::from_u8(v as u8))
            } else {
                None
            }
        })
    }
}
