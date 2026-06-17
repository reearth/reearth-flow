//! Attribute types moved to [`reearth_flow_common::attribute`] so that
//! lower-level crates (e.g. the geometry crate) can store feature attributes
//! without a dependency cycle through `reearth-flow-types`. Re-exported here
//! for backward compatibility.
//!
//! The CityGML conversion that used to live here (`From<nusamai_citygml::Value>`)
//! now lives as the free function
//! [`crate::conversion::nusamai::from_nusamai_citygml_value`], because the
//! orphan rule forbids implementing it for the relocated type outside the
//! defining crate. The previously-unused `rhai::Dynamic` and
//! `nusamai_citygml::schema` conversions were dropped.
pub use reearth_flow_common::attribute::{
    all_attribute_keys, Attribute, AttributeValue, Attributes,
};
