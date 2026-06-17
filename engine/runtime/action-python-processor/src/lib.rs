// TODO(new-geometry): remove after migration. Gating each ported action's
// `process`/`finish`/`start` leaves geometry-only imports and helpers unused
// under the flag; silence that noise (feature-scoped, so the default build keeps
// full lint coverage).
#![cfg_attr(feature = "new-geometry", allow(unused_imports, dead_code))]

mod mapping;
mod python_executor;

pub use mapping::ACTION_FACTORY_MAPPINGS;
pub use python_executor::PythonScriptProcessorFactory;
