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
