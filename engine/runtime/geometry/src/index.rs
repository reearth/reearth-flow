//! Dynamic mesh index width, shared by `PolygonMesh` and `TriangularMesh`.
//!
//! The index element type is chosen at construction time (not compile time) to
//! be the narrowest of `u8` / `u16` / `u32` that fits the largest value an array
//! must store. `u64` / `usize` are intentionally unsupported: a `u32::MAX`
//! vertex pool already exceeds the RAM of most target systems.

use serde::{Deserialize, Serialize};

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
