//! The Shapefile Reader action for the new geometry world.
//!
//! Self-contained: it shares no code with the old-world `shapefile` module, which
//! is compiled only without `new-geometry` and is to be deleted once the migration
//! is done.
//!
//! Not read: a shapefile outside a ZIP archive, or without a `.shp` and `.dbf`
//! sharing a stem; a table in UTF-16, or in a DOS code page (read as UTF-8 with a
//! warning); measures (discarded with a warning); a multipatch with `force2D`
//! set. Coordinates whose `.prj` PROJ cannot match carry no CRS.
mod archive;
mod geometry;
mod record;
mod source;

pub(crate) use source::ShapefileReaderFactory;
