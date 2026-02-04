//! glTF metadata encoding/decoding using EXT_structural_metadata extension

pub mod decode;
pub mod encode;

pub use decode::{
    extract_feature_properties, read_mesh_features, read_structural_metadata, PropertyData,
    PropertyTable, PropertyTables,
};
pub use encode::MetadataEncoder;

pub const ENUM_NO_DATA: u32 = 0;
pub const ENUM_NO_DATA_NAME: &str = "";
pub const FLOAT_NO_DATA: f64 = f64::MAX;

// Signed integer noData values - use MIN to avoid collision with valid data
pub const INT8_NO_DATA: i8 = i8::MIN;
pub const INT16_NO_DATA: i16 = i16::MIN;
pub const INT32_NO_DATA: i32 = i32::MIN;
pub const INT64_NO_DATA: i64 = i64::MIN;

// Unsigned integer noData values - use MAX to avoid collision with valid data
pub const UINT8_NO_DATA: u8 = u8::MAX;
pub const UINT16_NO_DATA: u16 = u16::MAX;
pub const UINT32_NO_DATA: u32 = u32::MAX;
pub const UINT64_NO_DATA: u64 = u64::MAX;
