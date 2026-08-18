//! The Shapefile Reader action for the new geometry world.
//!
//! Self-contained: it shares no code with the old-world `shapefile` module, which
//! is compiled only without `new-geometry` and is to be deleted once the migration
//! is done.
//!
//! What the reader does not take in:
//! - a shapefile outside a ZIP archive, or an archive with no `.shp` and `.dbf`
//!   sharing a stem;
//! - an attribute table in UTF-16, or in a DOS code page the web encodings leave
//!   out (read as UTF-8 with a warning);
//! - measures (M values), which are discarded with a warning;
//! - a multipatch with `force2D` set, which has no 2D counterpart;
//! - a `.prj` PROJ's database has no match for, whose coordinates carry no CRS.
mod archive;
mod geometry;
mod record;
mod source;

pub(crate) use source::ShapefileReaderFactory;
