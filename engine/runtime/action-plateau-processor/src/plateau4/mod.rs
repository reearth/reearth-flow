pub(crate) mod attribute_flattener;
pub(crate) mod building_installation_geometry_type_checker;
pub(crate) mod city_code_extractor;
pub(crate) mod citygml_mesh_builder;
pub(crate) mod composite_surface_continuity_filter;
pub(crate) mod destination_mesh_code_extractor;
pub(crate) mod errors;
pub(crate) mod face_extractor;
pub(crate) mod flooding_area_surface_generator;
pub(crate) mod gml_name_code_space_validator;
pub(crate) mod mapping;
pub(crate) mod max_lod_extractor;
mod profile;
pub(crate) mod tran_xlink_detector;
pub(crate) mod unmatched_xlink_strategy;
pub(crate) mod unshared_edge_detector;

// The PLATEAU4 profile is referenced from plateau4/mapping via `super::profile`.
// This re-export exists so the common check logic tests can reference it as
// `crate::plateau4::PLATEAU4`, and is only needed in test builds.
#[cfg(test)]
pub(crate) use profile::PLATEAU4;
