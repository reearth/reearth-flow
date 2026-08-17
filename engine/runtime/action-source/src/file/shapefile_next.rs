//! The Shapefile Reader action for the new geometry world.
//!
//! Self-contained: it shares no code with the old-world `shapefile` module, which
//! is compiled only without `new-geometry` and is to be deleted once the migration
//! is done.
mod archive;
mod geometry;
mod record;
mod source;

pub(crate) use source::ShapefileReaderFactory;
