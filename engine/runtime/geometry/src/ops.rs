//! Operations over the new-geometry types.
//!
//! Each operation is a trait implemented across the geometry types (the
//! `BoundingRect` pattern), with any heavy/unsafe machinery isolated in a
//! submodule.

pub mod reproject;
