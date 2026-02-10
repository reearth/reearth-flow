//! glTF metadata encoding/decoding using EXT_structural_metadata extension

pub mod decode;
pub mod encode;
pub mod int_type_selector;

pub use decode::{
    extract_feature_properties, read_mesh_features, read_structural_metadata, PropertyData,
    PropertyTable, PropertyTables,
};
pub use encode::MetadataEncoder;

pub const ENUM_NO_DATA: u32 = 0;
pub const ENUM_NO_DATA_NAME: &str = "";
pub const FLOAT_NO_DATA: f64 = f64::MAX;
