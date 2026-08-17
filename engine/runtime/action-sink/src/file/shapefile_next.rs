//! The Shapefile Writer action for the new geometry world.
//!
//! Self-contained: it shares no code with the old-world `shapefile` module, which
//! is compiled only without `new-geometry` and is to be deleted once the migration
//! is done.
pub(super) mod conversion;
pub(super) mod crs;
pub(super) mod null_shape;
pub(super) mod pipeline;
pub(super) mod shape;
pub(super) mod sink;

pub(crate) use sink::ShapefileWriterFactory;
