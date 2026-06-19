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

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, OnceLock};

use kiddo::ImmutableKdTree;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::coordinate::Coordinate;

/// Bit positions of the optional primary fields within a [`Segment`]'s
/// [`FieldMask`].
pub mod bit {
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
pub enum PositionEncoding {
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

impl PositionEncoding {
    /// Byte width of one XYZ position in `data`.
    #[inline]
    pub fn position_bytes(&self) -> u16 {
        match self {
            PositionEncoding::F64 => 24,
            PositionEncoding::F32 | PositionEncoding::ScaledI32 { .. } => 12,
        }
    }
}

/// Bit `i` set => optional field `i` is present in this segment.
pub type FieldMask = u16;
/// Byte offset of each field within the stride, indexed by bit.
pub type FieldOffsets = [u16; 9];

/// Typed column for user-defined per-point attributes (LAS Extra Bytes, PDAL
/// extra dims, etc.). Schema-based: every column has exactly `Segment::count`
/// entries. SoA layout: one `Vec<T>` per attribute, accessed one column at a
/// time.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AttributeColumn {
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

impl AttributeColumn {
    /// Number of entries in this column.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            AttributeColumn::UInt8(v) => v.len(),
            AttributeColumn::UInt16(v) => v.len(),
            AttributeColumn::UInt32(v) => v.len(),
            AttributeColumn::UInt64(v) => v.len(),
            AttributeColumn::Int8(v) => v.len(),
            AttributeColumn::Int16(v) => v.len(),
            AttributeColumn::Int32(v) => v.len(),
            AttributeColumn::Int64(v) => v.len(),
            AttributeColumn::Float32(v) => v.len(),
            AttributeColumn::Float64(v) => v.len(),
            AttributeColumn::String(v) => v.len(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether two columns hold the same variant (same element type).
    #[inline]
    pub fn same_type(&self, other: &AttributeColumn) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    /// Append another column of the same variant onto this one.
    ///
    /// # Panics
    /// Panics if the variants differ; callers (`merge`) check the schema first.
    pub fn extend(&mut self, other: AttributeColumn) {
        match (self, other) {
            (AttributeColumn::UInt8(a), AttributeColumn::UInt8(b)) => a.extend(b),
            (AttributeColumn::UInt16(a), AttributeColumn::UInt16(b)) => a.extend(b),
            (AttributeColumn::UInt32(a), AttributeColumn::UInt32(b)) => a.extend(b),
            (AttributeColumn::UInt64(a), AttributeColumn::UInt64(b)) => a.extend(b),
            (AttributeColumn::Int8(a), AttributeColumn::Int8(b)) => a.extend(b),
            (AttributeColumn::Int16(a), AttributeColumn::Int16(b)) => a.extend(b),
            (AttributeColumn::Int32(a), AttributeColumn::Int32(b)) => a.extend(b),
            (AttributeColumn::Int64(a), AttributeColumn::Int64(b)) => a.extend(b),
            (AttributeColumn::Float32(a), AttributeColumn::Float32(b)) => a.extend(b),
            (AttributeColumn::Float64(a), AttributeColumn::Float64(b)) => a.extend(b),
            (AttributeColumn::String(a), AttributeColumn::String(b)) => a.extend(b),
            _ => panic!("AttributeColumn::extend: mismatched column types"),
        }
    }
}

/// One acquisition source's points, carrying only the fields it actually has.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Segment {
    /// File / acquisition identifier; `None` = synthetic.
    pub(crate) source: Option<Arc<str>>,
    /// XYZ encoding; determines the first bytes of each stride.
    pub(crate) position: PositionEncoding,
    /// Which optional fields are present.
    pub(crate) fields: FieldMask,
    /// `stride = position_bytes + sum(size of each present field)`.
    pub(crate) stride: u16,
    /// `offsets[i]` = byte start of field `i` within the stride; 0 if absent.
    pub(crate) offsets: FieldOffsets,
    /// Packed little-endian AoS byte stream; `len = count * stride`. No
    /// alignment guarantee; read / written only via `from_le_bytes` /
    /// `to_le_bytes`.
    pub(crate) data: Vec<u8>,
    pub(crate) count: usize,
    /// User-defined columns; each column's `len() == count`.
    pub(crate) attributes: HashMap<String, AttributeColumn>,
}

impl Segment {
    /// Decode the GPS timestamp of point `i`, or `None` when the segment carries
    /// no timestamp field. Illustrates the decode-by-copy access model: a
    /// bounds-checked slice plus a length-checked `[u8; 8]` decode, no `unsafe`
    /// and no alignment precondition.
    pub fn timestamp(&self, i: usize) -> Option<f64> {
        if self.fields & (1 << bit::TIMESTAMP) == 0 {
            return None;
        }
        let off = i * self.stride as usize + self.offsets[bit::TIMESTAMP as usize] as usize;
        Some(f64::from_le_bytes(
            self.data[off..off + 8].try_into().unwrap(),
        ))
    }
}

/// Opaque spatial-query handle; packs `(segment_idx: u32, point_idx: u32)`.
/// Invalidated by any [`PointCloud::merge`] (the KD-tree reset is the signal).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointIndex(u64);

impl PointIndex {
    #[inline]
    pub fn new(segment_idx: u32, point_idx: u32) -> Self {
        PointIndex(((segment_idx as u64) << 32) | point_idx as u64)
    }

    #[inline]
    pub fn segment_idx(self) -> u32 {
        (self.0 >> 32) as u32
    }

    #[inline]
    pub fn point_idx(self) -> u32 {
        self.0 as u32
    }
}

/// A 3D point cloud: one or more acquisition [`Segment`]s sharing a frame, plus
/// a lazily-built global KD-tree.
#[derive(Serialize, Deserialize)]
pub struct PointCloud {
    /// Coordinate frame all segments are expressed in.
    pub(crate) coordinate: Coordinate,
    /// One segment inline (no heap allocation) is the common case.
    pub(crate) segments: SmallVec<[Segment; 1]>,
    /// Built lazily on first spatial query, or pre-built explicitly. Not part of
    /// the serialized form, and reset on any mutation. The kiddo alias
    /// `ImmutableKdTree<f64, 3>` expands to `<f64, u64, 3, 32>`: `f64` coords,
    /// `u64` content, 3 dimensions, bucket size 32.
    #[serde(skip)]
    pub(crate) kdtree: OnceLock<ImmutableKdTree<f64, 3>>,
}

impl PointCloud {
    /// Merge another point cloud in. An incoming segment whose source matches an
    /// existing one is concatenated; mismatched schema for the same source is a
    /// hard error. Anonymous (`source: None`) segments are always appended. The
    /// KD-tree cache is invalidated.
    pub fn merge(&mut self, other: PointCloud) {
        for incoming in other.segments {
            match &incoming.source {
                Some(name) => {
                    let existing = self
                        .segments
                        .iter_mut()
                        .find(|s| s.source.as_deref() == Some(name.as_ref()));
                    match existing {
                        Some(existing) => {
                            // Same source must have identical schema: position
                            // encoding, primary field mask, and attribute
                            // column names + types must all match.
                            assert_eq!(existing.position, incoming.position);
                            assert_eq!(existing.fields, incoming.fields);
                            assert!(schemas_match(&existing.attributes, &incoming.attributes));
                            existing.data.extend_from_slice(&incoming.data);
                            for (key, col) in incoming.attributes {
                                existing.attributes.get_mut(&key).unwrap().extend(col);
                            }
                            existing.count += incoming.count;
                        }
                        None => self.segments.push(incoming),
                    }
                }
                None => self.segments.push(incoming), // anonymous: always appended
            }
        }
        // Invalidated: new points may change the spatial extent.
        self.kdtree = OnceLock::new();
    }
}

/// Whether two attribute schemas have the same column names and element types.
pub fn schemas_match(
    a: &HashMap<String, AttributeColumn>,
    b: &HashMap<String, AttributeColumn>,
) -> bool {
    a.len() == b.len()
        && a.iter().all(|(key, col)| match b.get(key) {
            Some(other) => col.same_type(other),
            None => false,
        })
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
