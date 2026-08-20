// TODO(new-geometry): remove after migration. While actions are being ported,
// gating each `process`/`finish`/`start` leaves their geometry-only imports and
// helpers unused under the flag. Silence that noise here (scoped to the feature,
// so the default build keeps full lint coverage).
#![cfg_attr(feature = "new-geometry", allow(unused_imports, dead_code))]

pub(crate) mod errors;
pub(crate) mod feature_creator;
pub(crate) mod file;
pub mod mapping;
pub(crate) mod sql;
