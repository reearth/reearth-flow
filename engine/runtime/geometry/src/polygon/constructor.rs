//! Polygon constructors.
//!
//! A `Polygon` is built from *rings* — an exterior boundary and zero or more
//! interior holes, each a sequence of coordinates. There are two ways in, chosen
//! by how the *source format* delivers rings, not by how any current reader
//! happens to be written:
//!
//! * [`PolygonBuilder2D`] / [`PolygonBuilder3D`] — for formats that deliver rings
//!   one at a time, where a polygon's interiors are not all known up front. These
//!   are the streaming-native formats:
//!     - **CityGML / GML (XML)** — `quick_xml` is a pull parser and XML is
//!       inherently sequential: a `<gml:exterior>` ring is parsed before its
//!       `<gml:interior>` rings, so a streaming parse calls `set_exterior` then
//!       `push_interior` as each ring's `posList` is read.
//!     - **GeoPackage (WKB)** — ring data is read straight from a byte cursor, one
//!       self-describing ring after another, so rings are produced incrementally.
//!     - **Shapefile** — one shape record is a flat `Outer`/`Inner` ring list that
//!       may hold several polygons (an `Outer` starts a new one); the builder turns
//!       that walk into a polygon per exterior.
//!
//! * [`Polygon2D::from_rings`] / [`Polygon3D::from_rings`] — for formats whose
//!   parser materializes the whole polygon before you ever see it, so the exterior
//!   and all interiors are already in hand: **GeoJSON** (`Value::Polygon` is a
//!   fully-built `Vec<Vec<Position>>`) and **WKT / CSV** (`wkt`/`geo_types` collect
//!   every ring up front). A streaming builder would only re-feed an
//!   already-materialized structure, so the all-at-once form is the honest fit.
//!
//! Either way the rings are flattened into the leaf's CSR layout: exterior and
//! interiors concatenated into one `coords` buffer, with `interior_offsets`
//! recording each interior ring's start (the exterior is the prefix, so it carries
//! no offset of its own). Each ring is closed on the way in — a closing vertex
//! equal to the first is appended when the caller's ring is open — so the stored
//! invariant (every ring closed, first == last) holds however the source delivered
//! it; closing an already-closed ring is a no-op. Interior rings with no vertices
//! carry no geometry and are dropped.
//!
//! [`Polygon2D::from_raw_parts`] / [`Polygon3D::from_raw_parts`] take pre-flattened
//! CSR buffers directly and validate the layout invariants, returning [`Error`] on
//! violation rather than panicking.
//!
//! Constructed polygons are *bare*: no UV sets and no appearance. Attach an
//! appearance afterwards via [`Polygon2D::appearance_mut`] /
//! [`Polygon3D::appearance_mut`].

use std::marker::PhantomData;

use crate::coordinate::Coordinate;
use crate::error::Error;

use super::{Polygon2D, Polygon3D};

mod sealed {
    pub trait BuilderState {}
}

/// Type-level states for the polygon builders ([`PolygonBuilder2D`] /
/// [`PolygonBuilder3D`]). The state is a phantom type parameter, so the ordering
/// rules — exterior before any interior, set exactly once, present at `build` — are
/// enforced by the type checker rather than at runtime. The trait is sealed: these
/// are the only two states.
pub mod state {
    /// No exterior ring has been set yet. Only `set_exterior` is available.
    #[derive(Debug, Clone, Copy)]
    pub struct Empty;
    /// The exterior ring is set; `push_interior` and `build` are available, and
    /// `set_exterior` is not.
    #[derive(Debug, Clone, Copy)]
    pub struct HasExterior;
    impl super::sealed::BuilderState for Empty {}
    impl super::sealed::BuilderState for HasExterior {}
}

use sealed::BuilderState;
use state::{Empty, HasExterior};

impl Polygon2D {
    /// Build a 2D polygon from an exterior ring and interior holes, each a
    /// sequence of `[x, y]`. The result is pure 2D (no elevation, no allocation
    /// for `z`); for per-vertex elevation use [`Polygon2D::from_rings_with_elevation`].
    ///
    /// Rings are concatenated exterior-first and each is closed; empty interior
    /// rings are dropped.
    pub fn from_rings<E, I, R>(coordinate: Coordinate, exterior: E, interiors: I) -> Self
    where
        E: IntoIterator<Item = [f64; 2]>,
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = [f64; 2]>,
    {
        let (coords, interior_offsets) = flatten_rings::<2, _, _, _>(exterior, interiors);
        Self {
            coordinate,
            coords: coords.into_boxed_slice(),
            interior_offsets: interior_offsets.into_boxed_slice(),
            z: None,
            uv_sets: Vec::new(),
            appearance: None,
        }
    }

    /// Build a 2.5D polygon from rings of `[x, y, z]`: the `(x, y)` populate
    /// `coords` and the `z` the parallel elevation buffer. Use this for sources
    /// that carry elevation on an otherwise 2D footprint (e.g. a height-tagged
    /// shapefile or GeoPackage layer).
    pub fn from_rings_with_elevation<E, I, R>(
        coordinate: Coordinate,
        exterior: E,
        interiors: I,
    ) -> Self
    where
        E: IntoIterator<Item = [f64; 3]>,
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = [f64; 3]>,
    {
        let (xyz, interior_offsets) = flatten_rings::<3, _, _, _>(exterior, interiors);
        let mut coords = Vec::with_capacity(xyz.len());
        let mut z = Vec::with_capacity(xyz.len());
        for [x, y, zz] in xyz {
            coords.push([x, y]);
            z.push(zz);
        }
        Self {
            coordinate,
            coords: coords.into_boxed_slice(),
            interior_offsets: interior_offsets.into_boxed_slice(),
            z: Some(z.into_boxed_slice()),
            uv_sets: Vec::new(),
            appearance: None,
        }
    }

    /// Build directly from already-flattened CSR buffers, for callers that hold
    /// the leaf's exact layout (deserialization, slicing, geometry algorithms).
    ///
    /// The layout invariants are validated: `z`, when present, must be parallel to
    /// `coords`, and `interior_offsets` must be strictly increasing with each
    /// offset in `1..coords.len()` (every ring non-empty, exterior included).
    /// Violations return [`Error::InvalidGeometry`].
    pub fn from_raw_parts(
        coordinate: Coordinate,
        coords: Box<[[f64; 2]]>,
        interior_offsets: Box<[u32]>,
        z: Option<Box<[f64]>>,
    ) -> Result<Self, Error> {
        if let Some(z) = z.as_ref() {
            if z.len() != coords.len() {
                return Err(Error::invalid_geometry(format!(
                    "elevation buffer length {} does not match coordinate count {}",
                    z.len(),
                    coords.len()
                )));
            }
        }
        check_offsets(&interior_offsets, coords.len())?;
        Ok(Self {
            coordinate,
            coords,
            interior_offsets,
            z,
            uv_sets: Vec::new(),
            appearance: None,
        })
    }
}

impl Polygon3D {
    /// Build a 3D polygon from an exterior ring and interior holes, each a
    /// sequence of `[x, y, z]`.
    ///
    /// Rings are concatenated exterior-first and each is closed; empty interior
    /// rings are dropped.
    pub fn from_rings<E, I, R>(coordinate: Coordinate, exterior: E, interiors: I) -> Self
    where
        E: IntoIterator<Item = [f64; 3]>,
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = [f64; 3]>,
    {
        let (coords, interior_offsets) = flatten_rings::<3, _, _, _>(exterior, interiors);
        Self {
            coordinate,
            coords: coords.into_boxed_slice(),
            interior_offsets: interior_offsets.into_boxed_slice(),
            uv_sets: Vec::new(),
            appearance: None,
        }
    }

    /// Build directly from already-flattened CSR buffers. `interior_offsets` must
    /// be strictly increasing with each offset in `1..coords.len()`; violations
    /// return [`Error::InvalidGeometry`].
    pub fn from_raw_parts(
        coordinate: Coordinate,
        coords: Box<[[f64; 3]]>,
        interior_offsets: Box<[u32]>,
    ) -> Result<Self, Error> {
        check_offsets(&interior_offsets, coords.len())?;
        Ok(Self {
            coordinate,
            coords,
            interior_offsets,
            uv_sets: Vec::new(),
            appearance: None,
        })
    }
}

/// Incremental builder for a [`Polygon2D`], for sources that discover an exterior
/// ring first and its holes later (the streaming-native formats — CityGML/GML,
/// GeoPackage WKB, shapefile).
///
/// Ordering is a *typestate* machine: the `S` parameter tracks whether the exterior
/// is set, so the rules are enforced at compile time rather than checked at runtime.
/// [`new`](Self::new) yields a `PolygonBuilder2D<`[`Empty`](state::Empty)`>` exposing
/// only [`set_exterior`](Self::set_exterior); that consumes it and yields a
/// `PolygonBuilder2D<`[`HasExterior`](state::HasExterior)`>` exposing
/// [`push_interior`](Self::push_interior) and [`build`](Self::build). So an interior
/// before the exterior, a second exterior, or a build with no exterior simply do not
/// compile:
///
/// ```compile_fail
/// use reearth_flow_geometry::coordinate::Coordinate;
/// use reearth_flow_geometry::polygon::PolygonBuilder2D;
/// // No `build` on the `Empty` state.
/// let _ = PolygonBuilder2D::new(Coordinate::Euclidean).build();
/// ```
///
/// ```compile_fail
/// use reearth_flow_geometry::coordinate::Coordinate;
/// use reearth_flow_geometry::polygon::PolygonBuilder2D;
/// // No `push_interior` before `set_exterior`.
/// let _ = PolygonBuilder2D::new(Coordinate::Euclidean).push_interior([[0.0, 0.0]]);
/// ```
///
/// Produces a pure-2D polygon (no elevation).
#[derive(Debug, Clone)]
pub struct PolygonBuilder2D<S: BuilderState = Empty> {
    coordinate: Coordinate,
    coords: Vec<[f64; 2]>,
    interior_offsets: Vec<u32>,
    _state: PhantomData<S>,
}

impl PolygonBuilder2D<Empty> {
    /// Start an empty builder in `coordinate`, awaiting an exterior ring.
    pub fn new(coordinate: Coordinate) -> Self {
        Self {
            coordinate,
            coords: Vec::new(),
            interior_offsets: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Set the exterior ring from a sequence of `[x, y]`, advancing the builder to
    /// the `HasExterior` state. The ring is closed if open.
    pub fn set_exterior<R>(self, ring: R) -> PolygonBuilder2D<HasExterior>
    where
        R: IntoIterator<Item = [f64; 2]>,
    {
        let mut coords: Vec<[f64; 2]> = ring.into_iter().collect();
        if !coords.is_empty() {
            close_ring(&mut coords, 0);
        }
        PolygonBuilder2D {
            coordinate: self.coordinate,
            coords,
            interior_offsets: Vec::new(),
            _state: PhantomData,
        }
    }
}

impl PolygonBuilder2D<HasExterior> {
    /// Append an interior (hole) ring from a sequence of `[x, y]`. An empty ring is
    /// dropped.
    pub fn push_interior<R>(mut self, ring: R) -> Self
    where
        R: IntoIterator<Item = [f64; 2]>,
    {
        let start = self.coords.len();
        self.coords.extend(ring);
        if self.coords.len() == start {
            return self;
        }
        close_ring(&mut self.coords, start);
        self.interior_offsets.push(start as u32);
        self
    }

    /// Finish the polygon. Returns [`Error::InvalidGeometry`] only on a data-level
    /// problem the type system cannot rule out — e.g. interiors atop an empty
    /// exterior ring (see [`Polygon2D::from_raw_parts`]).
    pub fn build(self) -> Result<Polygon2D, Error> {
        Polygon2D::from_raw_parts(
            self.coordinate,
            self.coords.into_boxed_slice(),
            self.interior_offsets.into_boxed_slice(),
            None,
        )
    }
}

/// Incremental builder for a [`Polygon3D`]; the 3D counterpart of
/// [`PolygonBuilder2D`], with rings of `[x, y, z]` and the same typestate machine.
#[derive(Debug, Clone)]
pub struct PolygonBuilder3D<S: BuilderState = Empty> {
    coordinate: Coordinate,
    coords: Vec<[f64; 3]>,
    interior_offsets: Vec<u32>,
    _state: PhantomData<S>,
}

impl PolygonBuilder3D<Empty> {
    /// Start an empty builder in `coordinate`, awaiting an exterior ring.
    pub fn new(coordinate: Coordinate) -> Self {
        Self {
            coordinate,
            coords: Vec::new(),
            interior_offsets: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Set the exterior ring from a sequence of `[x, y, z]`, advancing the builder
    /// to the `HasExterior` state. The ring is closed if open.
    pub fn set_exterior<R>(self, ring: R) -> PolygonBuilder3D<HasExterior>
    where
        R: IntoIterator<Item = [f64; 3]>,
    {
        let mut coords: Vec<[f64; 3]> = ring.into_iter().collect();
        if !coords.is_empty() {
            close_ring(&mut coords, 0);
        }
        PolygonBuilder3D {
            coordinate: self.coordinate,
            coords,
            interior_offsets: Vec::new(),
            _state: PhantomData,
        }
    }
}

impl PolygonBuilder3D<HasExterior> {
    /// Append an interior (hole) ring from a sequence of `[x, y, z]`. An empty ring
    /// is dropped.
    pub fn push_interior<R>(mut self, ring: R) -> Self
    where
        R: IntoIterator<Item = [f64; 3]>,
    {
        let start = self.coords.len();
        self.coords.extend(ring);
        if self.coords.len() == start {
            return self;
        }
        close_ring(&mut self.coords, start);
        self.interior_offsets.push(start as u32);
        self
    }

    /// Finish the polygon. Returns [`Error::InvalidGeometry`] only on a data-level
    /// problem the type system cannot rule out (see [`Polygon3D::from_raw_parts`]).
    pub fn build(self) -> Result<Polygon3D, Error> {
        Polygon3D::from_raw_parts(
            self.coordinate,
            self.coords.into_boxed_slice(),
            self.interior_offsets.into_boxed_slice(),
        )
    }
}

/// Flatten an exterior ring and interior holes into the CSR `(coords,
/// interior_offsets)` pair shared by both polygon dimensions.
///
/// The exterior fills the prefix of `coords` (and is closed); each non-empty
/// interior ring is then appended, closed, and its start index recorded in
/// `interior_offsets`. Empty interior rings are skipped so no offset ever points
/// at a zero-length slice.
fn flatten_rings<const N: usize, E, I, R>(exterior: E, interiors: I) -> (Vec<[f64; N]>, Vec<u32>)
where
    E: IntoIterator<Item = [f64; N]>,
    I: IntoIterator<Item = R>,
    R: IntoIterator<Item = [f64; N]>,
{
    let mut coords: Vec<[f64; N]> = exterior.into_iter().collect();
    if coords.is_empty() {
        // No exterior ring: an empty polygon carries no holes, so we drop any and
        // never record an interior offset of 0 — the invalid state (a hole posing as
        // the exterior) that `from_raw_parts`'s `1..coords.len()` check rejects.
        return (coords, Vec::new());
    }
    close_ring(&mut coords, 0);
    let mut interior_offsets: Vec<u32> = Vec::new();
    for ring in interiors {
        let start = coords.len();
        coords.extend(ring);
        if coords.len() == start {
            continue;
        }
        close_ring(&mut coords, start);
        interior_offsets.push(start as u32);
    }
    (coords, interior_offsets)
}

/// Close the ring occupying `coords[start..]` by appending a copy of its first
/// vertex when it does not already end where it began. `coords[start..]` must be
/// non-empty.
fn close_ring<const N: usize>(coords: &mut Vec<[f64; N]>, start: usize) {
    let first = coords[start];
    let last = coords[coords.len() - 1];
    if first != last {
        coords.push(first);
    }
}

/// Validate `interior_offsets` against a `coords_len`-vertex buffer: strictly
/// increasing, with every offset in `1..coords_len` so the exterior prefix and
/// each interior ring are non-empty.
fn check_offsets(offsets: &[u32], coords_len: usize) -> Result<(), Error> {
    let mut lo = 1u32;
    for (i, &o) in offsets.iter().enumerate() {
        if o < lo || o as usize >= coords_len {
            return Err(Error::invalid_geometry(format!(
                "interior_offsets[{i}] = {o} is out of range (expected {lo}..{coords_len})"
            )));
        }
        lo = o + 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_rings_closes_open_exterior() {
        let p = Polygon2D::from_rings(
            Coordinate::Euclidean,
            [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            Vec::<Vec<[f64; 2]>>::new(),
        );
        assert_eq!(p.coords.len(), 4);
        assert_eq!(p.coords[0], p.coords[3]);
        assert!(p.interior_offsets.is_empty());
        assert!(p.z.is_none());
        assert_eq!(p.coordinate, Coordinate::Euclidean);
    }

    // An empty exterior yields an empty polygon with its holes dropped — never an
    // interior offset of 0, which `from_raw_parts` rejects.
    #[test]
    fn from_rings_empty_exterior_drops_holes() {
        let p = Polygon3D::from_rings(
            Coordinate::Euclidean,
            Vec::<[f64; 3]>::new(),
            vec![vec![[1.0, 1.0, 0.0], [2.0, 1.0, 0.0], [2.0, 2.0, 0.0]]],
        );
        assert!(p.coords.is_empty());
        assert!(p.interior_offsets.is_empty());
    }

    #[test]
    fn from_rings_keeps_closed_exterior() {
        let p = Polygon2D::from_rings(
            Coordinate::Euclidean,
            [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]],
            Vec::<Vec<[f64; 2]>>::new(),
        );
        assert_eq!(p.coords.len(), 4);
    }

    #[test]
    fn from_rings_with_one_hole() {
        let p = Polygon2D::from_rings(
            Coordinate::Euclidean,
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
            vec![vec![[1.0, 1.0], [2.0, 1.0], [2.0, 2.0]]],
        );
        // exterior: 4 open -> closed to 5; hole: 3 open -> closed to 4.
        assert_eq!(p.interior_offsets.as_ref(), &[5]);
        assert_eq!(p.coords.len(), 9);
        let ext = &p.coords[..p.interior_offsets[0] as usize];
        let hole = &p.coords[p.interior_offsets[0] as usize..];
        assert_eq!(ext.len(), 5);
        assert_eq!(hole.len(), 4);
        assert_eq!(hole[0], hole[hole.len() - 1]);
    }

    #[test]
    fn from_rings_drops_empty_interior() {
        let p = Polygon2D::from_rings(
            Coordinate::Euclidean,
            [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            vec![Vec::<[f64; 2]>::new()],
        );
        assert!(p.interior_offsets.is_empty());
        assert_eq!(p.coords.len(), 4);
    }

    #[test]
    fn from_rings_with_elevation_splits_z() {
        let p = Polygon2D::from_rings_with_elevation(
            Coordinate::Euclidean,
            [[0.0, 0.0, 10.0], [1.0, 0.0, 11.0], [0.0, 1.0, 12.0]],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let z = p.z.as_ref().expect("elevation present");
        assert_eq!(z.len(), p.coords.len());
        assert_eq!(p.coords[1], [1.0, 0.0]);
        assert_eq!(z[1], 11.0);
        // The appended closing vertex carries the first vertex's elevation.
        assert_eq!(z[3], 10.0);
    }

    #[test]
    fn from_rings_3d() {
        let p = Polygon3D::from_rings(
            Coordinate::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 1.0]],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        assert_eq!(p.coords.len(), 4);
        assert_eq!(p.coords[0], p.coords[3]);
        assert!(p.interior_offsets.is_empty());
    }

    #[test]
    fn from_raw_parts_stores_buffers() {
        let coords: Box<[[f64; 2]]> =
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]].into_boxed_slice();
        let p =
            Polygon2D::from_raw_parts(Coordinate::Euclidean, coords.clone(), Box::new([]), None)
                .expect("valid layout");
        assert_eq!(p.coords, coords);
        assert!(p.interior_offsets.is_empty());
        assert!(p.z.is_none());
        assert!(p.uv_sets.is_empty());
        assert!(p.appearance.is_none());
    }

    #[test]
    fn from_raw_parts_rejects_unparallel_z() {
        let coords: Box<[[f64; 2]]> = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]].into_boxed_slice();
        let err = Polygon2D::from_raw_parts(
            Coordinate::Euclidean,
            coords,
            Box::new([]),
            Some(vec![0.0, 0.0].into_boxed_slice()),
        )
        .unwrap_err();
        assert!(matches!(err, Error::InvalidGeometry(_)));
    }

    #[test]
    fn from_raw_parts_rejects_bad_offsets() {
        let coords: Box<[[f64; 3]]> = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ]
        .into_boxed_slice();
        // Offset 0 would leave the exterior empty.
        assert!(
            Polygon3D::from_raw_parts(Coordinate::Euclidean, coords.clone(), Box::new([0]))
                .is_err()
        );
        // Offset == coords.len() leaves a zero-length interior.
        assert!(
            Polygon3D::from_raw_parts(Coordinate::Euclidean, coords.clone(), Box::new([4]))
                .is_err()
        );
        // Non-increasing offsets.
        assert!(
            Polygon3D::from_raw_parts(Coordinate::Euclidean, coords, Box::new([2, 2])).is_err()
        );
    }

    // Ordering rules are enforced at compile time by the typestate, so there are no
    // runtime-rejection tests; see the `compile_fail` doctests on `PolygonBuilder2D`.
    #[test]
    fn builder_streams_rings() {
        let built = PolygonBuilder2D::new(Coordinate::Euclidean)
            .set_exterior([[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]])
            .push_interior([[1.0, 1.0], [2.0, 1.0], [2.0, 2.0]])
            .build()
            .unwrap();

        let batch = Polygon2D::from_rings(
            Coordinate::Euclidean,
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
            vec![vec![[1.0, 1.0], [2.0, 1.0], [2.0, 2.0]]],
        );
        assert_eq!(built, batch);
    }

    #[test]
    fn builder_streams_rings_3d() {
        let built = PolygonBuilder3D::new(Coordinate::Euclidean)
            .set_exterior([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 1.0]])
            .build()
            .unwrap();

        let batch = Polygon3D::from_rings(
            Coordinate::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 1.0]],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        assert_eq!(built, batch);
    }
}
