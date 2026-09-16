//! The Shapefile Writer action for the new geometry world.
//!
//! Self-contained: it shares no code with the old-world `shapefile` module, which
//! is compiled only without `new-geometry` and is to be deleted once the migration
//! is done.
//!
//! Left out, with a warning: point clouds, CSG trees, all but the first kind of a
//! collection mixing points, curves and areas, curves of fewer than two positions,
//! rings of fewer than three, and array, map and byte attribute values. No `.prj`
//! is written for positions from no CRS, several, or one with no ESRI WKT1 form;
//! a vertical CRS is never written. 3D meshes and solids are written as
//! multipatches, a solid's shells undistinguished; 2D meshes as polygons.
//! Measures are never written. Field names are cut to 11 bytes, text to 254 and
//! numbers to 15 decimal places.
pub(super) mod conversion;
pub(super) mod crs;
pub(super) mod null_shape;
pub(super) mod pipeline;
pub(super) mod shape;
pub(super) mod sink;

pub(crate) use sink::ShapefileWriterFactory;
