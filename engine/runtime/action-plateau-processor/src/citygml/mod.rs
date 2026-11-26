mod spec;
mod v2;
pub(crate) mod v3;
mod xml;

pub(crate) use spec::{CityGmlSpec, GML_DICTIONARY_NS};
pub(crate) use v2::CITYGML2;
pub(crate) use v3::CITYGML3;
pub(crate) use xml::gml_id;
