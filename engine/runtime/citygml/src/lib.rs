// TODO(new-geometry): remove after migration. The legacy `build_features` path's
// geometry-only imports and helpers go unused under the flag; silence that noise
// (feature-scoped, so the default build keeps full lint coverage).
#![cfg_attr(feature = "new-geometry", allow(unused_imports, dead_code))]

#[cfg(feature = "new-geometry")]
pub(crate) mod appearance;
pub(crate) mod codespace;
pub(crate) mod flatten;
#[cfg(not(feature = "new-geometry"))]
pub(crate) mod geometry;
#[cfg(feature = "new-geometry")]
#[path = "geometry_next.rs"]
pub(crate) mod geometry;
#[cfg(not(feature = "new-geometry"))]
pub mod parser;
#[cfg(feature = "new-geometry")]
#[path = "parser_next.rs"]
pub mod parser;
pub mod pipeline;
#[cfg(feature = "new-geometry")]
pub(crate) mod resolver;
#[cfg(feature = "new-geometry")]
mod srsname;
pub(crate) mod utils;
pub(crate) mod xlink;
