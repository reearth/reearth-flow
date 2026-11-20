//! PLATEAU 6 (CityGML 3.0) quality checks.
//!
//! Not checked yet: `grp:CityObjectGroup`. The objectlist reader drops the rows
//! of `21. 都市オブジェクトグループモデル`, since which packages a group should
//! be checked in is undecided.

mod building_usage_attribute_strategy;
#[cfg(feature = "new-geometry")]
mod errors;
mod mapping;
mod object_list_reader;
mod profile;
pub(crate) mod transportation_xlink_strategy;
mod unmatched_xlink_strategy;
#[cfg(feature = "new-geometry")]
mod unshared_edge_extractor;

pub(crate) use mapping::ACTION_FACTORY_MAPPINGS;

// The PLATEAU6 profile is referenced from plateau6/mapping via `super::profile`.
// This re-export exists so the common check logic tests can reference it as
// `crate::plateau6::PLATEAU6`, and is only needed in test builds.
#[cfg(test)]
pub(crate) use profile::PLATEAU6;
