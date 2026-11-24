//! PointCloud leaf.
//!
//! Distinct from a `Collection` of `Point`s: designed for large unstructured 3D
//! spatial samples (LiDAR, photogrammetry, depth cameras) where per-point
//! primary attributes are first-class. The type is 3D only.
//!
//! The primary-field buffer is a safe abstraction, not a reinterpreted blob:
//! `PointBuffer::data` is a packed little-endian byte stream with no alignment
//! guarantee, so every read decodes through `from_le_bytes` and every write
//! through `to_le_bytes`. No `unsafe` is used on the field-access path, so a miscomputed
//! offset is a bounds panic, never undefined behavior.

use std::fmt;
use std::sync::{Arc, OnceLock};

use indexmap::IndexMap;
use kiddo::ImmutableKdTree;
use serde::{Deserialize, Serialize};

use crate::coordinate::CoordinateFrame;
use crate::ops::UnsupportedOperation;

use file::PointFile;

mod constructor;
#[cfg(not(feature = "debug-geom-feature-write"))]
mod feature_write;
mod file;
mod las;
mod ops;
#[cfg(feature = "new-geometry")]
mod validation;

/// Bit positions of the optional primary fields within a [`PointBuffer`]'s
/// [`FieldMask`].
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

/// XYZ storage format: a buffer-level switch, invisible to the public API.
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

/// Bit `i` set => optional field `i` is present in this buffer.
type FieldMask = u16;
/// Byte offset of each field within the stride, indexed by bit.
type FieldOffsets = [u16; 9];

/// Bytes of each optional field, indexed by bit; `0` = not yet supported.
///
/// - RGB: `[u16; 3]`
/// - INTENSITY: `u16`
/// - TIMESTAMP: `f64` (GPS time)
/// - CLASSIFICATION: `u8` (ASPRS class)
/// - RETURN_INFO: `u8`, return number in bits 0-3, number of returns in bits 4-7
/// - SCAN_ANGLE: `i16`, in units of 0.006 degrees
/// - POINT_SOURCE_ID: `u16`
const FIELD_SIZE: [u16; 9] = [6, 2, 0, 8, 1, 1, 2, 0, 2];

impl PositionEncoding {
    /// Bytes of one position.
    fn size(&self) -> u16 {
        match self {
            PositionEncoding::F64 => 24,
            PositionEncoding::F32 | PositionEncoding::ScaledI32 { .. } => 12,
        }
    }
}

/// Stride and per-field offsets for `fields` after a position of `position`,
/// fields in bit order.
///
/// # Panics
/// If `fields` has a bit with no size in [`FIELD_SIZE`].
fn layout(position: &PositionEncoding, fields: FieldMask) -> (u16, FieldOffsets) {
    let mut offsets = [0; 9];
    let mut stride = position.size();
    for (b, size) in FIELD_SIZE.iter().enumerate() {
        if fields & (1 << b) != 0 {
            assert!(*size != 0, "point-cloud field bit {b} is not supported");
            offsets[b] = stride;
            stride += size;
        }
    }
    (stride, offsets)
}

/// Typed column for user-defined per-point attributes (LAS Extra Bytes, PDAL
/// extra dims, etc.). Schema-based: every column has exactly `PointBuffer::count`
/// entries. SoA layout: one `Vec<T>` per attribute, accessed one column at a
/// time.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

#[cfg(not(feature = "debug-geom-feature-write"))]
impl AttributeColumn {
    /// Entries in this column, which must equal the buffer's `count`.
    fn len(&self) -> usize {
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
}

/// Where a point cloud's points are held.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum Data {
    /// Points held in memory.
    InMemory(PointBuffer),
    /// Points left in a LAS / LAZ file, read on use.
    File(PointFile),
}

impl Data {
    /// The in-memory points, or an error naming `operation` if the points are
    /// still in a file.
    fn buffer(&self, operation: &'static str) -> Result<&PointBuffer, UnsupportedOperation> {
        match self {
            Data::InMemory(buffer) => Ok(buffer),
            Data::File(_) => Err(not_loaded(operation)),
        }
    }

    /// Mutable [`Data::buffer`].
    #[cfg(feature = "new-geometry")]
    fn buffer_mut(
        &mut self,
        operation: &'static str,
    ) -> Result<&mut PointBuffer, UnsupportedOperation> {
        match self {
            Data::InMemory(buffer) => Ok(buffer),
            Data::File(_) => Err(not_loaded(operation)),
        }
    }
}

#[cfg(test)]
impl Data {
    /// The in-memory points; panics on file data.
    fn as_buffer(&self) -> &PointBuffer {
        match self {
            Data::InMemory(buffer) => buffer,
            Data::File(file) => panic!("points are in file {}", file.uri()),
        }
    }
}

impl From<PointBuffer> for Data {
    fn from(buffer: PointBuffer) -> Self {
        Data::InMemory(buffer)
    }
}

/// The error for an operation that needs points still in a file.
fn not_loaded(operation: &'static str) -> UnsupportedOperation {
    UnsupportedOperation {
        geometry: "PointCloud (file-backed)",
        operation,
    }
}

/// Points held in memory, carrying only the fields they actually have.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct PointBuffer {
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

impl PointBuffer {
    /// A buffer of `count` zeroed points with `fields` present and no
    /// attribute columns.
    fn zeroed(
        source: Option<Arc<str>>,
        position: PositionEncoding,
        fields: FieldMask,
        count: usize,
    ) -> Self {
        let (stride, offsets) = layout(&position, fields);
        PointBuffer {
            source,
            position,
            fields,
            stride,
            offsets,
            data: vec![0; count * stride as usize],
            count,
            attributes: IndexMap::new(),
        }
    }

    /// An empty buffer with `fields` present and room reserved for
    /// `capacity` points, or `None` if that much memory cannot be reserved.
    /// Reserved memory is not touched until points are added.
    fn with_capacity(
        source: Option<Arc<str>>,
        position: PositionEncoding,
        fields: FieldMask,
        capacity: usize,
    ) -> Option<Self> {
        let (stride, offsets) = layout(&position, fields);
        let mut data = Vec::new();
        data.try_reserve_exact(capacity.checked_mul(stride as usize)?)
            .ok()?;
        Some(PointBuffer {
            source,
            position,
            fields,
            stride,
            offsets,
            data,
            count: 0,
            attributes: IndexMap::new(),
        })
    }

    /// Append `n` zeroed points.
    fn extend_zeroed(&mut self, n: usize) {
        self.count += n;
        self.data.resize(self.count * self.stride as usize, 0);
    }

    /// Whether field `bit` is present.
    fn has(&self, bit: u16) -> bool {
        self.fields & (1 << bit) != 0
    }

    /// The bytes of field `bit` of point `i`.
    #[cfg_attr(feature = "debug-geom-feature-write", allow(dead_code))]
    fn field<const N: usize>(&self, bit: u16, i: usize) -> [u8; N] {
        debug_assert!(self.has(bit) && FIELD_SIZE[bit as usize] as usize == N);
        let at = i * self.stride as usize + self.offsets[bit as usize] as usize;
        self.data[at..at + N].try_into().unwrap()
    }

    /// Overwrite field `bit` of point `i`.
    fn set_field<const N: usize>(&mut self, bit: u16, i: usize, bytes: [u8; N]) {
        debug_assert!(self.has(bit) && FIELD_SIZE[bit as usize] as usize == N);
        let at = i * self.stride as usize + self.offsets[bit as usize] as usize;
        self.data[at..at + N].copy_from_slice(&bytes);
    }

    /// Overwrite the position bytes of point `i`.
    fn set_position(&mut self, i: usize, bytes: &[u8]) {
        debug_assert_eq!(bytes.len(), self.position.size() as usize);
        let at = i * self.stride as usize;
        self.data[at..at + bytes.len()].copy_from_slice(bytes);
    }
}

/// A 3D point cloud: its points in a frame, plus a lazily-built KD-tree. A
/// point's position is its stored position times `scale` plus `translation`.
#[cfg_attr(feature = "debug-geom-feature-write", derive(Serialize, Deserialize))]
pub struct PointCloud {
    /// Coordinate frame the points are expressed in.
    frame: CoordinateFrame,
    data: Data,
    /// Per-axis factor on every stored position.
    scale: [f64; 3],
    /// Added to every stored position after `scale`.
    translation: [f64; 3],
    /// Built lazily on first spatial query, or pre-built explicitly. Not part of
    /// the serialized form, and reset on any mutation. The kiddo alias
    /// `ImmutableKdTree<f64, 3>` has `f64` coords, `u32` content, 3 dimensions,
    /// bucket size 32.
    #[cfg_attr(feature = "debug-geom-feature-write", serde(skip))]
    kdtree: OnceLock<ImmutableKdTree<f64, 3>>,
}

impl PointCloud {
    /// The coordinate frame the points are expressed in.
    #[inline]
    pub fn frame(&self) -> &CoordinateFrame {
        &self.frame
    }

    /// A cloud of `data` in `frame` with no scale or translation.
    fn new(frame: CoordinateFrame, data: Data) -> Self {
        PointCloud {
            frame,
            data,
            scale: [1.0; 3],
            translation: [0.0; 3],
            kdtree: OnceLock::new(),
        }
    }

    /// A stored position with `scale` and `translation` applied.
    fn place(&self, p: [f64; 3]) -> [f64; 3] {
        std::array::from_fn(|i| p[i] * self.scale[i] + self.translation[i])
    }

    /// Every point's position, or an error naming `operation` if the points
    /// are still in a file.
    fn positions(
        &self,
        operation: &'static str,
    ) -> Result<impl Iterator<Item = [f64; 3]> + '_, UnsupportedOperation> {
        let buffer = self.data.buffer(operation)?;
        Ok(ops::buffer_positions(buffer).map(|p| self.place(p)))
    }
}

impl Clone for PointCloud {
    fn clone(&self) -> Self {
        // The KD-tree is a derived cache, rebuilt lazily; a clone starts fresh.
        PointCloud {
            frame: self.frame.clone(),
            data: self.data.clone(),
            scale: self.scale,
            translation: self.translation,
            kdtree: OnceLock::new(),
        }
    }
}

impl PartialEq for PointCloud {
    fn eq(&self, other: &Self) -> bool {
        // The KD-tree cache is not part of the value.
        self.frame == other.frame
            && self.data == other.data
            && self.scale == other.scale
            && self.translation == other.translation
    }
}

impl fmt::Debug for PointCloud {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PointCloud")
            .field("frame", &self.frame)
            .field("data", &self.data)
            .field("scale", &self.scale)
            .field("translation", &self.translation)
            .field("kdtree", &self.kdtree.get().map(|_| "<built>"))
            .finish()
    }
}

#[cfg(feature = "new-geometry")]
crate::unsupported!(PointCloud: Footprint);

// A cloud of samples has no surface to divide.
#[cfg(feature = "new-geometry")]
crate::unsupported!(PointCloud: DivideByGrid);

// A cloud of samples has no surface between its points.
#[cfg(feature = "new-geometry")]
crate::no_area!(PointCloud);

// Positions may be stored packed at a fixed precision, which rounding would not
// preserve.
#[cfg(feature = "new-geometry")]
crate::unsupported!(PointCloud: RoundCoordinates);

// A cloud of samples has no order along a line to number its points by.
#[cfg(feature = "new-geometry")]
crate::unsupported!(PointCloud: SelectVertices);

crate::unsupported!(
    PointCloud: Triangulate,
    Reproject,
    ConvertFrame,
    ForceTwoDimension,
    RemoveAppearance,
    CountHoles,
    ExtractHoles,
    Coerce
);

// A cloud is nothing but coordinates. A file's count comes from its header.
#[cfg(feature = "new-geometry")]
impl crate::ops::CountVertices for PointCloud {
    fn count_vertices(&self) -> crate::error::Result<usize> {
        match &self.data {
            Data::InMemory(buffer) => Ok(buffer.count),
            Data::File(file) => Ok(file.header()?.count as usize),
        }
    }
}

// Positions have no extent, so nothing bounds them.
impl crate::ops::ExtractBoundary for PointCloud {
    fn extract_boundary(&self) -> Result<crate::ops::Boundary, crate::ops::UnsupportedOperation> {
        Ok(crate::ops::Boundary::EMPTY)
    }
}

#[cfg(feature = "new-geometry")]
crate::unsupported!(PointCloud: Extrude);
#[cfg(feature = "new-geometry")]
crate::unsupported!(PointCloud: DecomposeIntoEdgeConnectedParts, DecomposeIntoManifolds);
