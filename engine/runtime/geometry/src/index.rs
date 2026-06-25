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
pub(crate) enum IndexBuffer<const N: usize> {
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

/// One of the three index widths an [`IndexBuffer`] can store, ordered narrowest
/// to widest. Used to name a width when constructing a buffer.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum IndexWidth {
    U8,
    U16,
    U32,
}

impl IndexWidth {
    /// Narrowest width that can store `value`.
    pub(crate) fn for_value(value: u32) -> Self {
        if value <= u8::MAX as u32 {
            IndexWidth::U8
        } else if value <= u16::MAX as u32 {
            IndexWidth::U16
        } else {
            IndexWidth::U32
        }
    }

    /// Narrowest width that can store every component of `element`.
    fn for_element<const N: usize>(element: &[u32; N]) -> Self {
        Self::for_value(element.iter().copied().max().unwrap_or(0))
    }

    /// Position in the narrowest-to-widest order, for comparing widths.
    fn rank(self) -> u8 {
        match self {
            IndexWidth::U8 => 0,
            IndexWidth::U16 => 1,
            IndexWidth::U32 => 2,
        }
    }
}

/// Reallocate `buf` into `target` when `target` is wider than the current width.
/// Narrowing is a no-op: the constructors only ever promote.
fn promote<const N: usize>(buf: &mut IndexBuffer<N>, target: IndexWidth) {
    if target.rank() <= buf.width().rank() {
        return;
    }
    let old = std::mem::replace(buf, IndexBuffer::U8(Vec::new()));
    *buf = match (old, target) {
        (IndexBuffer::U8(v), IndexWidth::U16) => {
            IndexBuffer::U16(v.into_iter().map(|e| e.map(|x| x as u16)).collect())
        }
        (IndexBuffer::U8(v), IndexWidth::U32) => {
            IndexBuffer::U32(v.into_iter().map(|e| e.map(|x| x as u32)).collect())
        }
        (IndexBuffer::U16(v), IndexWidth::U32) => {
            IndexBuffer::U32(v.into_iter().map(|e| e.map(|x| x as u32)).collect())
        }
        (old, _) => old,
    };
}

/// Allocate `len` elements without initializing them, write each item from `iter`
/// (each component mapped through `f`) by raw pointer with no per-element bounds
/// check, then set the length to the number actually written.
///
/// Private: the only public entry point to this is the single unsafe interface
/// [`IndexBuffer::from_exact_unchecked`], which dispatches the concrete element
/// type into this generic body.
///
/// # Safety
/// The caller must ensure `iter` yields **at most** `len` items (yielding more
/// writes out of bounds). Yielding fewer simply produces a shorter buffer.
unsafe fn fill_uninit<const N: usize, T, F>(
    len: usize,
    iter: impl IntoIterator<Item = [u32; N]>,
    f: F,
) -> Vec<[T; N]>
where
    F: Fn(u32) -> T + Copy,
{
    let mut v: Vec<[T; N]> = Vec::with_capacity(len);
    let start = v.as_mut_ptr();
    let end = start.add(len);
    let mut dst = start;
    for element in iter {
        debug_assert!(
            dst < end,
            "iterator yielded more than the {len} items reserved"
        );
        dst.write(element.map(f));
        dst = dst.add(1);
    }
    v.set_len(dst.offset_from(start) as usize);
    v
}

impl<const N: usize> IndexBuffer<N> {
    /// The width this buffer currently stores its indices in.
    pub(crate) fn width(&self) -> IndexWidth {
        match self {
            IndexBuffer::U8(_) => IndexWidth::U8,
            IndexBuffer::U16(_) => IndexWidth::U16,
            IndexBuffer::U32(_) => IndexWidth::U32,
        }
    }

    /// The number of `N`-tuples stored (e.g. triangle count for `IndexBuffer<3>`,
    /// corner count for `IndexBuffer<1>`).
    pub(crate) fn len(&self) -> usize {
        match self {
            IndexBuffer::U8(v) => v.len(),
            IndexBuffer::U16(v) => v.len(),
            IndexBuffer::U32(v) => v.len(),
        }
    }

    /// Build from index elements, choosing the narrowest width that fits them all.
    ///
    /// Width-agnostic: storage starts at `u8` and fattens to `u16` then `u32` only
    /// when an element exceeds the current width — so the common small-mesh case
    /// stays `u8` and never pays for wider storage. A large mesh whose size is
    /// unknown here reallocates on each promotion (at most twice), the same linear
    /// cost its `Vec` growth pays anyway.
    pub(crate) fn from_indices(iter: impl IntoIterator<Item = [u32; N]>) -> Self {
        Self::with_min_width(IndexWidth::U8, None, iter)
    }

    /// Build starting at `width`, promoting to a wider one if an element needs it;
    /// the result width is therefore `max(width, needed)`. Never panics: a `width`
    /// too narrow for the data is widened, a `width` wider than necessary is kept
    /// (costing storage but not correctness). `size_hint`, when given, presizes the
    /// initial allocation.
    pub(crate) fn with_min_width(
        width: IndexWidth,
        size_hint: Option<usize>,
        iter: impl IntoIterator<Item = [u32; N]>,
    ) -> Self {
        let iter = iter.into_iter();
        let cap = size_hint.unwrap_or_else(|| iter.size_hint().0);
        let mut buf = match width {
            IndexWidth::U8 => IndexBuffer::U8(Vec::with_capacity(cap)),
            IndexWidth::U16 => IndexBuffer::U16(Vec::with_capacity(cap)),
            IndexWidth::U32 => IndexBuffer::U32(Vec::with_capacity(cap)),
        };
        for element in iter {
            promote(&mut buf, IndexWidth::for_element(&element));
            match &mut buf {
                IndexBuffer::U8(v) => v.push(element.map(|x| x as u8)),
                IndexBuffer::U16(v) => v.push(element.map(|x| x as u16)),
                IndexBuffer::U32(v) => v.push(element),
            }
        }
        buf
    }

    /// Build at exactly `width`, panicking if an element does not fit. Unlike
    /// [`with_min_width`](Self::with_min_width) this never promotes — use it when
    /// the width is known and a mismatch is a bug worth surfacing. `size_hint`,
    /// when given, presizes the allocation.
    ///
    /// # Panics
    /// If any element has a component larger than `width` can store.
    pub(crate) fn with_exact_width(
        width: IndexWidth,
        size_hint: Option<usize>,
        iter: impl IntoIterator<Item = [u32; N]>,
    ) -> Self {
        let iter = iter.into_iter();
        let cap = size_hint.unwrap_or_else(|| iter.size_hint().0);
        match width {
            IndexWidth::U8 => {
                let mut v = Vec::with_capacity(cap);
                for element in iter {
                    v.push(element.map(|x| {
                        assert!(x <= u8::MAX as u32, "index {x} does not fit in u8");
                        x as u8
                    }));
                }
                IndexBuffer::U8(v)
            }
            IndexWidth::U16 => {
                let mut v = Vec::with_capacity(cap);
                for element in iter {
                    v.push(element.map(|x| {
                        assert!(x <= u16::MAX as u32, "index {x} does not fit in u16");
                        x as u16
                    }));
                }
                IndexBuffer::U16(v)
            }
            IndexWidth::U32 => {
                let mut v = Vec::with_capacity(cap);
                v.extend(iter);
                IndexBuffer::U32(v)
            }
        }
    }

    /// Build at exactly `width` and exactly `len` elements, allocating uninitialized
    /// and filling from `iter` by raw pointer with no bounds checks — the fastest,
    /// most dangerous path.
    ///
    /// # Safety
    /// The caller must ensure both:
    /// - every element fits in `width` (a component that does not is silently
    ///   truncated), and
    /// - `iter` yields **at most** `len` items (yielding more writes out of bounds,
    ///   which is undefined behavior).
    pub(crate) unsafe fn from_exact_unchecked(
        width: IndexWidth,
        len: usize,
        iter: impl IntoIterator<Item = [u32; N]>,
    ) -> Self {
        match width {
            IndexWidth::U8 => IndexBuffer::U8(fill_uninit(len, iter, |x| x as u8)),
            IndexWidth::U16 => IndexBuffer::U16(fill_uninit(len, iter, |x| x as u16)),
            IndexWidth::U32 => IndexBuffer::U32(fill_uninit(len, iter, |x| x)),
        }
    }
}

impl<const N: usize> FromIterator<[u32; N]> for IndexBuffer<N> {
    /// Collects into the narrowest fitting width; see [`from_indices`](Self::from_indices).
    fn from_iter<I: IntoIterator<Item = [u32; N]>>(iter: I) -> Self {
        Self::from_indices(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u8s<const N: usize>(buf: &IndexBuffer<N>) -> &[[u8; N]] {
        match buf {
            IndexBuffer::U8(v) => v,
            _ => panic!("expected U8, got {:?}", buf.width()),
        }
    }

    #[test]
    fn from_indices_picks_u8_for_small_values() {
        let buf = IndexBuffer::from_indices([[0u32], [5], [255]]);
        assert_eq!(buf.width(), IndexWidth::U8);
        assert_eq!(u8s(&buf), &[[0], [5], [255]]);
    }

    #[test]
    fn from_indices_promotes_and_preserves_earlier_values() {
        let buf = IndexBuffer::from_indices([[1u32], [2], [300]]);
        assert_eq!(buf.width(), IndexWidth::U16);
        match &buf {
            IndexBuffer::U16(v) => assert_eq!(v, &[[1], [2], [300]]),
            _ => panic!("expected U16"),
        }
    }

    #[test]
    fn from_indices_promotes_to_u32() {
        let buf = IndexBuffer::from_indices([[1u32], [70_000]]);
        assert_eq!(buf.width(), IndexWidth::U32);
        match &buf {
            IndexBuffer::U32(v) => assert_eq!(v, &[[1], [70_000]]),
            _ => panic!("expected U32"),
        }
    }

    #[test]
    fn from_indices_promotes_once_for_triangles() {
        let buf = IndexBuffer::from_indices([[0u32, 1, 2], [3, 4, 70_000]]);
        assert_eq!(buf.width(), IndexWidth::U32);
        match &buf {
            IndexBuffer::U32(v) => assert_eq!(v, &[[0, 1, 2], [3, 4, 70_000]]),
            _ => panic!("expected U32"),
        }
    }

    #[test]
    fn with_min_width_keeps_wider_floor() {
        let buf = IndexBuffer::with_min_width(IndexWidth::U16, None, [[0u32], [5]]);
        assert_eq!(buf.width(), IndexWidth::U16);
    }

    #[test]
    fn with_min_width_promotes_above_floor() {
        let buf = IndexBuffer::with_min_width(IndexWidth::U16, Some(2), [[5u32], [70_000]]);
        assert_eq!(buf.width(), IndexWidth::U32);
    }

    #[test]
    fn with_exact_width_keeps_stated_width() {
        let buf = IndexBuffer::with_exact_width(IndexWidth::U32, None, [[0u32], [5]]);
        assert_eq!(buf.width(), IndexWidth::U32);
    }

    #[test]
    #[should_panic(expected = "does not fit in u8")]
    fn with_exact_width_panics_on_overflow() {
        let _ = IndexBuffer::with_exact_width(IndexWidth::U8, None, [[0u32], [256]]);
    }

    #[test]
    fn from_exact_unchecked_fills_buffer() {
        let buf =
            unsafe { IndexBuffer::from_exact_unchecked(IndexWidth::U16, 3, [[0u32], [1], [400]]) };
        match &buf {
            IndexBuffer::U16(v) => assert_eq!(v, &[[0], [1], [400]]),
            _ => panic!("expected U16"),
        }
    }

    #[test]
    fn collect_uses_minimal_width() {
        let buf: IndexBuffer<3> = [[0u32, 1, 2], [2, 3, 4]].into_iter().collect();
        assert_eq!(buf.width(), IndexWidth::U8);
    }
}
