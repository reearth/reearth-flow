//! The Shapefile Writer action for the new geometry world.
//!
//! Self-contained: it shares no code with the old-world `shapefile` module, which
//! is compiled only without `new-geometry` and is to be deleted once the migration
//! is done.
//!
//! What the writer leaves out, each with a warning:
//! - point clouds and CSG trees, which have no shapefile counterpart;
//! - a collection mixing points, curves and areas, of which only the first kind is
//!   written;
//! - a curve of fewer than two positions and a ring of fewer than three;
//! - attribute values that are arrays, maps or bytes;
//! - a `.prj` for a file whose positions come from no CRS, from several, or from
//!   one with no ESRI WKT1 form; a vertical CRS is never written.
//!
//! Meshes and solids are written as their faces, so the multipatch shape type is
//! never produced. Measures (M values) are never written. Field names are cut to
//! 11 bytes, text to 254 bytes and numbers to 15 decimal places.
pub(super) mod conversion;
pub(super) mod crs;
pub(super) mod null_shape;
pub(super) mod pipeline;
pub(super) mod shape;
pub(super) mod sink;

pub(crate) use sink::ShapefileWriterFactory;
