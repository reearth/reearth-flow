// TODO(new-geometry): remove after migration. Gating each ported action's
// `process`/`finish`/`start` leaves geometry-only imports and helpers unused
// under the flag; silence that noise (feature-scoped, so the default build keeps
// full lint coverage).
#![cfg_attr(feature = "new-geometry", allow(unused_imports, dead_code))]

mod atlas;
pub mod echo;
pub mod errors;
pub mod file;
pub mod mapping;
pub mod noop;
mod output;
pub mod schema;
mod zip_eq_logged;

pub use output::SinkOutput;

mod sandbox;
pub use sandbox::{ensure_under, SandboxError};
