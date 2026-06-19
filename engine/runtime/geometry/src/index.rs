//! Dynamic mesh index width, shared by `PolygonMesh` and `TriangularMesh`.
//!
//! The index element type is chosen at construction time (not compile time) to
//! be the narrowest of `u8` / `u16` / `u32` that fits the largest value an array
//! must store. `u64` / `usize` are intentionally unsupported: a `u32::MAX`
//! vertex pool already exceeds the RAM of most target systems.

use serde::{Deserialize, Serialize};

/// Names the chosen index width. The container's discriminant ([`IndexBuffer`])
/// is the tag; there is no separate width field to fall out of sync.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexType {
    U8,
    U16,
    U32,
}

impl IndexType {
    /// Pick the narrowest index type that can store `max`.
    #[inline]
    pub fn for_max_value(max: usize) -> IndexType {
        match max {
            m if m <= u8::MAX as usize => IndexType::U8,
            m if m <= u16::MAX as usize => IndexType::U16,
            _ => IndexType::U32, // hard ceiling
        }
    }
}

/// Width-erased index storage. `N` is the stride: 3 for triangles, 1 for the
/// scalar arrays (`[Idx; 1]` is layout-identical to `Idx`).
///
/// `serde` only implements its array traits for lengths `0..=32`, so the derive
/// cannot prove `[Idx; N]: Serialize` for a fully generic `N`. The explicit
/// `bound` propagates the per-width array bounds; they hold for the only widths
/// used in practice (`N = 1` and `N = 3`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(bound(
    serialize = "[u8; N]: Serialize, [u16; N]: Serialize, [u32; N]: Serialize",
    deserialize = "[u8; N]: Deserialize<'de>, [u16; N]: Deserialize<'de>, [u32; N]: Deserialize<'de>"
))]
pub enum IndexBuffer<const N: usize> {
    U8(Vec<[u8; N]>),
    U16(Vec<[u16; N]>),
    U32(Vec<[u32; N]>),
}

impl<const N: usize> IndexBuffer<N> {
    /// The runtime width tag of this buffer.
    #[inline]
    pub fn index_type(&self) -> IndexType {
        match self {
            IndexBuffer::U8(_) => IndexType::U8,
            IndexBuffer::U16(_) => IndexType::U16,
            IndexBuffer::U32(_) => IndexType::U32,
        }
    }

    /// Number of stride-`N` entries.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            IndexBuffer::U8(v) => v.len(),
            IndexBuffer::U16(v) => v.len(),
            IndexBuffer::U32(v) => v.len(),
        }
    }

    /// Whether the buffer holds no entries.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The concrete integer types an [`IndexBuffer`] can store. Implemented for
/// `u8` / `u16` / `u32` so hot algorithms can run generically over a concrete
/// width selected by [`dispatch_index!`].
///
/// A trait and an enum cannot share a name (both live in the type namespace),
/// so the static path is `trait MeshIndex` while the runtime tag stays
/// `enum IndexType`.
pub trait MeshIndex: Copy + Ord {
    fn to_usize(self) -> usize;
    fn from_usize(i: usize) -> Self;
}

impl MeshIndex for u8 {
    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }
    #[inline]
    fn from_usize(i: usize) -> Self {
        i as u8
    }
}

impl MeshIndex for u16 {
    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }
    #[inline]
    fn from_usize(i: usize) -> Self {
        i as u16
    }
}

impl MeshIndex for u32 {
    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }
    #[inline]
    fn from_usize(i: usize) -> Self {
        i as u32
    }
}

/// Monomorphize a body over the concrete index width of an [`IndexBuffer`].
///
/// The body must be a thin call into a generic function, never an inlined
/// algorithm, so the 3-way expansion stays a stub. Pass a *reference* to the
/// buffer; match ergonomics then bind the slice as `&Vec<[Idx; N]>`:
///
/// ```ignore
/// fn is_connected(&self) -> bool {
///     dispatch_index!(&self.indices, |Idx, tris| self.is_connected_impl::<Idx>(tris))
/// }
/// ```
///
/// Algorithms that read two arrays of independently-chosen width nest the
/// macro, giving 3x3 = 9 monomorphizations of one generic impl.
#[macro_export]
macro_rules! dispatch_index {
    ($buf:expr, |$Idx:ident, $slice:ident| $body:expr) => {
        match $buf {
            $crate::index::IndexBuffer::U8(v) => {
                type $Idx = u8;
                let $slice = v;
                $body
            }
            $crate::index::IndexBuffer::U16(v) => {
                type $Idx = u16;
                let $slice = v;
                $body
            }
            $crate::index::IndexBuffer::U32(v) => {
                type $Idx = u32;
                let $slice = v;
                $body
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_max_value_picks_narrowest_width() {
        assert_eq!(IndexType::for_max_value(0), IndexType::U8);
        assert_eq!(IndexType::for_max_value(u8::MAX as usize), IndexType::U8);
        assert_eq!(
            IndexType::for_max_value(u8::MAX as usize + 1),
            IndexType::U16
        );
        assert_eq!(IndexType::for_max_value(u16::MAX as usize), IndexType::U16);
        assert_eq!(
            IndexType::for_max_value(u16::MAX as usize + 1),
            IndexType::U32
        );
    }

    #[test]
    fn dispatch_runs_generic_arm() {
        fn sum<Idx: MeshIndex>(slice: &[[Idx; 1]]) -> usize {
            slice.iter().map(|[i]| i.to_usize()).sum()
        }
        let buf: IndexBuffer<1> = IndexBuffer::U8(vec![[1u8], [2u8], [3u8]]);
        let total = dispatch_index!(&buf, |Idx, s| sum::<Idx>(s));
        assert_eq!(total, 6);
        assert_eq!(buf.index_type(), IndexType::U8);
    }
}
