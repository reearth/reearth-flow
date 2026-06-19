//! PointCloud leaf.
//!
//! Distinct from a `Collection` of `Point`s: designed for large unstructured 3D
//! spatial samples (LiDAR, photogrammetry, depth cameras) where per-point
//! primary attributes are first-class. The type is 3D only.
//!
//! The primary-field buffer is a safe abstraction, not a reinterpreted blob:
//! `data` is a packed little-endian byte stream with no alignment guarantee, so
//! every read decodes through `from_le_bytes` and every write through
//! `to_le_bytes`. No `unsafe` is used on the field-access path, so a miscomputed
//! offset is a bounds panic, never undefined behavior.

use std::fmt;
use std::sync::{Arc, OnceLock};

use indexmap::IndexMap;
use kiddo::ImmutableKdTree;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::coordinate::Coordinate;

/// Bit positions of the optional primary fields within a [`Segment`]'s
/// [`FieldMask`]. Private to this module: the full field-bit layout, not all of
/// which are referenced until the per-field accessors land.
#[allow(dead_code)]
mod bit {
    pub const RGB: u16 = 0;
    pub const INTENSITY: u16 = 1;
    pub const NORMAL: u16 = 2;
    pub const TIMESTAMP: u16 = 3;
    pub const CLASSIFICATION: u16 = 4;
    pub const RETURN_INFO: u16 = 5;
    pub const SCAN_ANGLE: u16 = 6;
    pub const RING: u16 = 7;
    pub const POINT_SOURCE_ID: u16 = 8;
}

/// XYZ storage format: a segment-level switch, invisible to the public API.
/// All public accessors decode to `f64` regardless of the variant stored.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum PositionEncoding {
    /// `[f64; 3]`, 24 bytes; georeferenced / post-transform.
    F64,
    /// `[f32; 3]`, 12 bytes; local CRS only (~3 cm error at 500 km).
    F32,
    /// `[i32; 3]`, 12 bytes; native LAS/COPC encoding.
    ScaledI32 {
        /// Per-axis scale factor (e.g. 0.001 = 1 mm).
        scale: [f64; 3],
        /// Per-axis offset.
        offset: [f64; 3],
    },
}

/// Bit `i` set => optional field `i` is present in this segment.
type FieldMask = u16;
/// Byte offset of each field within the stride, indexed by bit.
type FieldOffsets = [u16; 9];

/// Typed column for user-defined per-point attributes (LAS Extra Bytes, PDAL
/// extra dims, etc.). Schema-based: every column has exactly `Segment::count`
/// entries. SoA layout: one `Vec<T>` per attribute, accessed one column at a
/// time.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum AttributeColumn {
    UInt8(Vec<u8>),
    UInt16(Vec<u16>),
    UInt32(Vec<u32>),
    UInt64(Vec<u64>),
    Int8(Vec<i8>),
    Int16(Vec<i16>),
    Int32(Vec<i32>),
    Int64(Vec<i64>),
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    /// `None` = null / not yet assigned.
    String(Vec<Option<Arc<str>>>),
}
/// One acquisition source's points, carrying only the fields it actually has.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Segment {
    /// File / acquisition identifier; `None` = synthetic.
    source: Option<Arc<str>>,
    /// XYZ encoding; determines the first bytes of each stride.
    position: PositionEncoding,
    /// Which optional fields are present.
    fields: FieldMask,
    /// `stride = position_bytes + sum(size of each present field)`.
    stride: u16,
    /// `offsets[i]` = byte start of field `i` within the stride; 0 if absent.
    offsets: FieldOffsets,
    /// Packed little-endian AoS byte stream; `len = count * stride`. No
    /// alignment guarantee; read / written only via `from_le_bytes` /
    /// `to_le_bytes`.
    data: Vec<u8>,
    count: usize,
    /// User-defined columns; each column's `len() == count`. `IndexMap` rather
    /// than `HashMap` so the serialized column order is deterministic
    /// (insertion order), keeping the intermediate form byte-for-byte stable.
    attributes: IndexMap<String, AttributeColumn>,
}

/// A 3D point cloud: one or more acquisition [`Segment`]s sharing a frame, plus
/// a lazily-built global KD-tree.
#[derive(Serialize, Deserialize)]
pub struct PointCloud {
    /// Coordinate frame all segments are expressed in.
    coordinate: Coordinate,
    /// One segment inline (no heap allocation) is the common case.
    segments: SmallVec<[Segment; 1]>,
    /// Built lazily on first spatial query, or pre-built explicitly. Not part of
    /// the serialized form, and reset on any mutation. The kiddo alias
    /// `ImmutableKdTree<f64, 3>` expands to `<f64, u64, 3, 32>`: `f64` coords,
    /// `u64` content, 3 dimensions, bucket size 32.
    #[serde(skip)]
    kdtree: OnceLock<ImmutableKdTree<f64, 3>>,
}

impl Clone for PointCloud {
    fn clone(&self) -> Self {
        // The KD-tree is a derived cache, rebuilt lazily; a clone starts fresh.
        PointCloud {
            coordinate: self.coordinate.clone(),
            segments: self.segments.clone(),
            kdtree: OnceLock::new(),
        }
    }
}

impl PartialEq for PointCloud {
    fn eq(&self, other: &Self) -> bool {
        // The KD-tree cache is not part of the value.
        self.coordinate == other.coordinate && self.segments == other.segments
    }
}

impl fmt::Debug for PointCloud {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PointCloud")
            .field("coordinate", &self.coordinate)
            .field("segments", &self.segments)
            .field("kdtree", &self.kdtree.get().map(|_| "<built>"))
            .finish()
    }
}
