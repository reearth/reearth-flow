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
//! no offset of its own). Rings are stored **verbatim** — a ring is *not* closed,
//! so a malformed open ring (first != last) is preserved as-is for a later
//! validation phase to flag, never silently repaired. Interior rings with no
//! vertices carry no geometry and are dropped.
//!
//! [`Polygon2D::from_raw_parts`] / [`Polygon3D::from_raw_parts`] take pre-flattened
//! CSR buffers directly and validate the layout invariants, returning [`Error`] on
//! violation rather than panicking.
//!
//! Constructed polygons are *bare*: no appearance. Appearance is
//! attached afterwards (CityGML binds it by `gml:id` in a later pass than the
//! geometry) via the validated [`Polygon2D::set_appearance`] /
//! [`Polygon2D::set_two_sided_appearance`] setters below — or the raw
//! [`Polygon2D::appearance_mut`] escape hatch.

use std::marker::PhantomData;

use std::collections::BTreeMap;

use crate::appearance::{
    append_theme, single_channel_uv, validate_uv_coupling, Appearance, ChannelId, FaceBinding,
    Material, MaterialIndex, Side, ThemeId, UvSet, UvSource,
};
use crate::coordinate::CoordinateFrame;
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
    /// Rings are concatenated exterior-first and stored verbatim — *not* closed, so
    /// an open ring (first != last) is left as-is for later validation; empty
    /// interior rings are dropped.
    pub fn from_rings<E, I, R>(frame: CoordinateFrame, exterior: E, interiors: I) -> Self
    where
        E: IntoIterator<Item = [f64; 2]>,
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = [f64; 2]>,
    {
        let (coords, interior_offsets) = flatten_rings::<2, _, _, _>(exterior, interiors);
        Self {
            frame,
            coords: coords.into_boxed_slice(),
            interior_offsets: interior_offsets.into_boxed_slice(),
            z: None,
            appearance: None,
        }
    }

    /// Build a 2.5D polygon from rings of `[x, y, z]`: the `(x, y)` populate
    /// `coords` and the `z` the parallel elevation buffer. Use this for sources
    /// that carry elevation on an otherwise 2D footprint (e.g. a height-tagged
    /// shapefile or GeoPackage layer).
    pub fn from_rings_with_elevation<E, I, R>(
        frame: CoordinateFrame,
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
            frame,
            coords: coords.into_boxed_slice(),
            interior_offsets: interior_offsets.into_boxed_slice(),
            z: Some(z.into_boxed_slice()),
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
        frame: CoordinateFrame,
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
            frame,
            coords,
            interior_offsets,
            z,
            appearance: None,
        })
    }
}

impl Polygon3D {
    /// Build a 3D polygon from an exterior ring and interior holes, each a
    /// sequence of `[x, y, z]`.
    ///
    /// Rings are concatenated exterior-first and stored verbatim — *not* closed, so
    /// an open ring (first != last) is left as-is for later validation; empty
    /// interior rings are dropped.
    pub fn from_rings<E, I, R>(frame: CoordinateFrame, exterior: E, interiors: I) -> Self
    where
        E: IntoIterator<Item = [f64; 3]>,
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = [f64; 3]>,
    {
        let (coords, interior_offsets) = flatten_rings::<3, _, _, _>(exterior, interiors);
        Self {
            frame,
            coords: coords.into_boxed_slice(),
            interior_offsets: interior_offsets.into_boxed_slice(),
            appearance: None,
        }
    }

    /// Build directly from already-flattened CSR buffers. `interior_offsets` must
    /// be strictly increasing with each offset in `1..coords.len()`; violations
    /// return [`Error::InvalidGeometry`].
    pub fn from_raw_parts(
        frame: CoordinateFrame,
        coords: Box<[[f64; 3]]>,
        interior_offsets: Box<[u32]>,
    ) -> Result<Self, Error> {
        check_offsets(&interior_offsets, coords.len())?;
        Ok(Self {
            frame,
            coords,
            interior_offsets,
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
/// use reearth_flow_geometry::coordinate::CoordinateFrame;
/// use reearth_flow_geometry::polygon::PolygonBuilder2D;
/// // No `build` on the `Empty` state.
/// let _ = PolygonBuilder2D::new(CoordinateFrame::Euclidean).build();
/// ```
///
/// ```compile_fail
/// use reearth_flow_geometry::coordinate::CoordinateFrame;
/// use reearth_flow_geometry::polygon::PolygonBuilder2D;
/// // No `push_interior` before `set_exterior`.
/// let _ = PolygonBuilder2D::new(CoordinateFrame::Euclidean).push_interior([[0.0, 0.0]]);
/// ```
///
/// Produces a pure-2D polygon (no elevation).
#[derive(Debug, Clone)]
pub struct PolygonBuilder2D<S: BuilderState = Empty> {
    frame: CoordinateFrame,
    coords: Vec<[f64; 2]>,
    interior_offsets: Vec<u32>,
    _state: PhantomData<S>,
}

impl PolygonBuilder2D<Empty> {
    /// Start an empty builder in `frame`, awaiting an exterior ring.
    pub fn new(frame: CoordinateFrame) -> Self {
        Self {
            frame,
            coords: Vec::new(),
            interior_offsets: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Set the exterior ring from a sequence of `[x, y]`, advancing the builder to
    /// the `HasExterior` state. The ring is stored verbatim — not closed.
    pub fn set_exterior<R>(self, ring: R) -> PolygonBuilder2D<HasExterior>
    where
        R: IntoIterator<Item = [f64; 2]>,
    {
        PolygonBuilder2D {
            frame: self.frame,
            coords: ring.into_iter().collect(),
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
        self.interior_offsets.push(start as u32);
        self
    }

    /// Finish the polygon. Returns [`Error::InvalidGeometry`] only on a data-level
    /// problem the type system cannot rule out — e.g. interiors atop an empty
    /// exterior ring (see [`Polygon2D::from_raw_parts`]).
    pub fn build(self) -> Result<Polygon2D, Error> {
        Polygon2D::from_raw_parts(
            self.frame,
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
    frame: CoordinateFrame,
    coords: Vec<[f64; 3]>,
    interior_offsets: Vec<u32>,
    _state: PhantomData<S>,
}

impl PolygonBuilder3D<Empty> {
    /// Start an empty builder in `frame`, awaiting an exterior ring.
    pub fn new(frame: CoordinateFrame) -> Self {
        Self {
            frame,
            coords: Vec::new(),
            interior_offsets: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Set the exterior ring from a sequence of `[x, y, z]`, advancing the builder
    /// to the `HasExterior` state. The ring is stored verbatim — not closed.
    pub fn set_exterior<R>(self, ring: R) -> PolygonBuilder3D<HasExterior>
    where
        R: IntoIterator<Item = [f64; 3]>,
    {
        PolygonBuilder3D {
            frame: self.frame,
            coords: ring.into_iter().collect(),
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
        self.interior_offsets.push(start as u32);
        self
    }

    /// Finish the polygon. Returns [`Error::InvalidGeometry`] only on a data-level
    /// problem the type system cannot rule out (see [`Polygon3D::from_raw_parts`]).
    pub fn build(self) -> Result<Polygon3D, Error> {
        Polygon3D::from_raw_parts(
            self.frame,
            self.coords.into_boxed_slice(),
            self.interior_offsets.into_boxed_slice(),
        )
    }
}

/// Flatten an exterior ring and interior holes into the CSR `(coords,
/// interior_offsets)` pair shared by both polygon dimensions.
///
/// The exterior fills the prefix of `coords`; each non-empty interior ring is then
/// appended and its start index recorded in `interior_offsets`. Rings are stored
/// verbatim — not closed — so a malformed open ring is preserved for later
/// validation. Empty interior rings are skipped so no offset points at a
/// zero-length slice.
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
    let mut interior_offsets: Vec<u32> = Vec::new();
    for ring in interiors {
        let start = coords.len();
        coords.extend(ring);
        if coords.len() == start {
            continue;
        }
        interior_offsets.push(start as u32);
    }
    (coords, interior_offsets)
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

/// One side's shading for a polygon: a single material and the UV its textured
/// maps sample, one entry per UV channel the material references. `uv` is empty
/// for a colour-only material (no maps); each entry is an `Explicit` array
/// parallel to the polygon's `coords`, or a retained `WorldToTexture` matrix.
#[derive(Clone, Debug, PartialEq)]
pub struct PolygonFace {
    pub material: Material,
    pub uv: BTreeMap<ChannelId, UvSource>,
}

impl PolygonFace {
    /// A face whose textured map (if any) samples the default UV channel — the
    /// dominant single-`ParameterizedTexture` case.
    pub fn single(material: Material, uv: Option<UvSource>) -> Self {
        Self {
            material,
            uv: single_channel_uv(uv),
        }
    }
}

impl Polygon2D {
    /// Attach a single-theme, single-material, front-only appearance — the
    /// dominant CityGML case (one `ParameterizedTexture` / `X3DMaterial` per
    /// surface). See [`PolygonFace`] for the `uv` contract.
    ///
    /// Additive: call once per theme to build a multi-theme appearance; the first
    /// theme added becomes the default. For a distinct back side use
    /// [`set_two_sided_appearance`](Self::set_two_sided_appearance).
    ///
    /// Returns `Err` (leaving the polygon unchanged) if the theme is already set
    /// or any invariant is breached.
    pub fn set_appearance(
        &mut self,
        theme: ThemeId,
        material: Material,
        uv: Option<UvSource>,
    ) -> Result<(), Error> {
        add_polygon_theme(
            self.coords.len(),
            &mut self.appearance,
            theme,
            PolygonFace::single(material, uv),
            None,
        )
    }

    /// Attach a two-sided appearance for one theme: a distinct `front` and `back`
    /// face (CityGML `side="+"` / `side="-"`). Each face is still a single
    /// material. Additive like [`set_appearance`](Self::set_appearance).
    pub fn set_two_sided_appearance(
        &mut self,
        theme: ThemeId,
        front: PolygonFace,
        back: PolygonFace,
    ) -> Result<(), Error> {
        add_polygon_theme(
            self.coords.len(),
            &mut self.appearance,
            theme,
            front,
            Some(back),
        )
    }
}

impl Polygon3D {
    /// Attach a single-theme, single-material, front-only appearance. See
    /// [`Polygon2D::set_appearance`].
    pub fn set_appearance(
        &mut self,
        theme: ThemeId,
        material: Material,
        uv: Option<UvSource>,
    ) -> Result<(), Error> {
        add_polygon_theme(
            self.coords.len(),
            &mut self.appearance,
            theme,
            PolygonFace::single(material, uv),
            None,
        )
    }

    /// Attach a two-sided appearance for one theme. See
    /// [`Polygon2D::set_two_sided_appearance`].
    pub fn set_two_sided_appearance(
        &mut self,
        theme: ThemeId,
        front: PolygonFace,
        back: PolygonFace,
    ) -> Result<(), Error> {
        add_polygon_theme(
            self.coords.len(),
            &mut self.appearance,
            theme,
            front,
            Some(back),
        )
    }
}

/// Add one theme's appearance to a single-face polygon's `appearance`.
/// `corner_count` is the polygon's coordinate count. The `front` face is required;
/// `back` adds a distinct back-side material/UV. Shared by the 2D and 3D setters
/// so the invariants live in one place.
///
/// Additive: errors (leaving the polygon unchanged) if `theme` is already set or
/// any face breaches the material/UV-coupling or UV-length invariants. The first
/// theme added becomes the default.
fn add_polygon_theme(
    corner_count: usize,
    appearance: &mut Option<Appearance>,
    theme: ThemeId,
    front: PolygonFace,
    back: Option<PolygonFace>,
) -> Result<(), Error> {
    // Validate both faces into scratch buffers first, so an invalid back face
    // leaves the polygon untouched (`push_face` mutates as it validates).
    let mut materials: Vec<Material> = Vec::new();
    let mut new_uv_sets: Vec<UvSet> = Vec::new();
    let front_binding = push_face(
        &mut materials,
        &mut new_uv_sets,
        corner_count,
        Side::Front,
        front,
    )?;
    let back_binding = match back {
        Some(face) => Some(push_face(
            &mut materials,
            &mut new_uv_sets,
            corner_count,
            Side::Back,
            face,
        )?),
        None => None,
    };

    append_theme(
        appearance,
        theme,
        materials,
        front_binding,
        back_binding,
        new_uv_sets,
    )
}

/// Validate one face, push its material into the palette and (if any) its UV set,
/// and return the `Uniform` binding pointing at the just-added material.
fn push_face(
    materials: &mut Vec<Material>,
    uv_sets: &mut Vec<UvSet>,
    corner_count: usize,
    side: Side,
    face: PolygonFace,
) -> Result<FaceBinding, Error> {
    validate_uv_coupling(&face.material.referenced_channels(), &face.uv, corner_count)?;

    for (channel, uv) in face.uv {
        uv_sets.push(UvSet { side, channel, uv });
    }

    let index = u32::try_from(materials.len())
        .ok()
        .and_then(MaterialIndex::new)
        .ok_or_else(|| Error::invalid_appearance("material palette too large"))?;
    materials.push(face.material);
    Ok(FaceBinding::Uniform(index))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    // The exterior is stored verbatim: an open ring is NOT auto-closed, so the
    // first != last data bug survives for a later validation phase.
    #[test]
    fn from_rings_stores_exterior_verbatim() {
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            Vec::<Vec<[f64; 2]>>::new(),
        );
        assert_eq!(p.coords.len(), 3);
        assert_ne!(p.coords[0], p.coords[p.coords.len() - 1]);
        assert!(p.interior_offsets.is_empty());
        assert!(p.z.is_none());
        assert_eq!(p.frame, CoordinateFrame::Euclidean);
    }

    // An empty exterior yields an empty polygon with its holes dropped — never an
    // interior offset of 0, which `from_raw_parts` rejects.
    #[test]
    fn from_rings_empty_exterior_drops_holes() {
        let p = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            Vec::<[f64; 3]>::new(),
            vec![vec![[1.0, 1.0, 0.0], [2.0, 1.0, 0.0], [2.0, 2.0, 0.0]]],
        );
        assert!(p.coords.is_empty());
        assert!(p.interior_offsets.is_empty());
    }

    #[test]
    fn from_rings_keeps_closed_exterior() {
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]],
            Vec::<Vec<[f64; 2]>>::new(),
        );
        assert_eq!(p.coords.len(), 4);
    }

    #[test]
    fn from_rings_with_one_hole() {
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
            vec![vec![[1.0, 1.0], [2.0, 1.0], [2.0, 2.0]]],
        );
        // Stored verbatim: exterior 4, hole 3 — no closing vertices appended.
        assert_eq!(p.interior_offsets.as_ref(), &[4]);
        assert_eq!(p.coords.len(), 7);
        let ext = &p.coords[..p.interior_offsets[0] as usize];
        let hole = &p.coords[p.interior_offsets[0] as usize..];
        assert_eq!(ext.len(), 4);
        assert_eq!(hole.len(), 3);
    }

    #[test]
    fn from_rings_drops_empty_interior() {
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            vec![Vec::<[f64; 2]>::new()],
        );
        assert!(p.interior_offsets.is_empty());
        assert_eq!(p.coords.len(), 3);
    }

    #[test]
    fn from_rings_with_elevation_splits_z() {
        let p = Polygon2D::from_rings_with_elevation(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 10.0], [1.0, 0.0, 11.0], [0.0, 1.0, 12.0]],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let z = p.z.as_ref().expect("elevation present");
        assert_eq!(p.coords.len(), 3);
        assert_eq!(z.len(), p.coords.len());
        assert_eq!(p.coords[1], [1.0, 0.0]);
        assert_eq!(z[1], 11.0);
    }

    #[test]
    fn from_rings_3d() {
        let p = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 1.0]],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        assert_eq!(p.coords.len(), 3);
        assert_ne!(p.coords[0], p.coords[p.coords.len() - 1]);
        assert!(p.interior_offsets.is_empty());
    }

    #[test]
    fn from_raw_parts_stores_buffers() {
        let coords: Box<[[f64; 2]]> =
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]].into_boxed_slice();
        let p = Polygon2D::from_raw_parts(
            CoordinateFrame::Euclidean,
            coords.clone(),
            Box::new([]),
            None,
        )
        .expect("valid layout");
        assert_eq!(p.coords, coords);
        assert!(p.interior_offsets.is_empty());
        assert!(p.z.is_none());
        assert!(p.appearance.is_none());
    }

    #[test]
    fn from_raw_parts_rejects_unparallel_z() {
        let coords: Box<[[f64; 2]]> = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]].into_boxed_slice();
        let err = Polygon2D::from_raw_parts(
            CoordinateFrame::Euclidean,
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
        assert!(Polygon3D::from_raw_parts(
            CoordinateFrame::Euclidean,
            coords.clone(),
            Box::new([0])
        )
        .is_err());
        // Offset == coords.len() leaves a zero-length interior.
        assert!(Polygon3D::from_raw_parts(
            CoordinateFrame::Euclidean,
            coords.clone(),
            Box::new([4])
        )
        .is_err());
        // Non-increasing offsets.
        assert!(
            Polygon3D::from_raw_parts(CoordinateFrame::Euclidean, coords, Box::new([2, 2]))
                .is_err()
        );
    }

    // Ordering rules are enforced at compile time by the typestate, so there are no
    // runtime-rejection tests; see the `compile_fail` doctests on `PolygonBuilder2D`.
    #[test]
    fn builder_streams_rings() {
        let built = PolygonBuilder2D::new(CoordinateFrame::Euclidean)
            .set_exterior([[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]])
            .push_interior([[1.0, 1.0], [2.0, 1.0], [2.0, 2.0]])
            .build()
            .unwrap();

        let batch = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
            vec![vec![[1.0, 1.0], [2.0, 1.0], [2.0, 2.0]]],
        );
        assert_eq!(built, batch);
    }

    #[test]
    fn builder_streams_rings_3d() {
        let built = PolygonBuilder3D::new(CoordinateFrame::Euclidean)
            .set_exterior([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 1.0]])
            .build()
            .unwrap();

        let batch = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 1.0]],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        assert_eq!(built, batch);
    }

    // ── Appearance setters ──

    /// A 3-corner triangle polygon (open ring), so `coords.len() == 3`.
    fn triangle() -> Polygon3D {
        Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            Vec::<Vec<[f64; 3]>>::new(),
        )
    }

    #[test]
    fn textured_material_with_matching_uv_is_accepted() {
        let mut p = triangle();
        p.set_appearance(theme("rgb"), textured(), Some(uv(3)))
            .unwrap();

        let app = p.appearance().as_ref().unwrap();
        assert_eq!(app.materials().len(), 1);
        assert_eq!(app.themes().len(), 1);
        assert_eq!(*app.default_theme(), theme("rgb"));
        assert!(matches!(app.themes()[0].front, FaceBinding::Uniform(_)));
        assert!(app.themes()[0].back.is_none());
        assert_eq!(app.themes()[0].theme, theme("rgb"));
        assert_eq!(app.themes()[0].uv_sets.len(), 1);
        assert_eq!(app.themes()[0].uv_sets[0].side, Side::Front);
    }

    #[test]
    fn bare_material_with_no_uv_is_accepted() {
        let mut p = triangle();
        p.set_appearance(theme("rgb"), bare(), None).unwrap();
        let app = p.appearance().as_ref().unwrap();
        assert_eq!(app.materials().len(), 1);
        assert!(app.themes()[0].uv_sets.is_empty());
    }

    #[test]
    fn textured_material_without_uv_is_rejected() {
        let mut p = triangle();
        let err = p
            .set_appearance(theme("rgb"), textured(), None)
            .unwrap_err();
        assert!(matches!(err, Error::InvalidAppearance(_)));
        assert!(p.appearance().is_none(), "left unchanged on error");
    }

    #[test]
    fn bare_material_with_uv_is_rejected() {
        let mut p = triangle();
        let err = p
            .set_appearance(theme("rgb"), bare(), Some(uv(3)))
            .unwrap_err();
        assert!(matches!(err, Error::InvalidAppearance(_)));
    }

    #[test]
    fn uv_length_mismatch_is_rejected() {
        let mut p = triangle();
        // 4 UVs against a 3-corner polygon.
        let err = p
            .set_appearance(theme("rgb"), textured(), Some(uv(4)))
            .unwrap_err();
        assert!(matches!(err, Error::InvalidAppearance(_)));
    }

    #[test]
    fn world_to_texture_uv_skips_length_check() {
        use crate::appearance::TexMatrix;
        let mut p = triangle();
        let m = UvSource::WorldToTexture(TexMatrix([[0.0; 4]; 3]));
        p.set_appearance(theme("rgb"), textured(), Some(m)).unwrap();
        assert_eq!(
            p.appearance().as_ref().unwrap().themes()[0].uv_sets.len(),
            1
        );
    }

    #[test]
    fn first_theme_added_becomes_default() {
        let mut p = triangle();
        p.set_appearance(theme("infrared"), bare(), None).unwrap();
        p.set_appearance(theme("rgb"), bare(), None).unwrap();

        let app = p.appearance().as_ref().unwrap();
        assert_eq!(*app.default_theme(), theme("infrared"));
        assert_eq!(app.themes().len(), 2);
    }

    #[test]
    fn duplicate_theme_is_rejected() {
        let mut p = triangle();
        p.set_appearance(theme("rgb"), bare(), None).unwrap();
        let err = p.set_appearance(theme("rgb"), bare(), None).unwrap_err();
        assert!(matches!(err, Error::InvalidAppearance(_)));
    }

    #[test]
    fn two_themes_build_a_two_entry_palette() {
        let mut p = triangle();
        p.set_appearance(theme("rgb"), textured(), Some(uv(3)))
            .unwrap();
        p.set_appearance(theme("infrared"), bare(), None).unwrap();

        let app = p.appearance().as_ref().unwrap();
        assert_eq!(app.materials().len(), 2);
        assert_eq!(app.themes().len(), 2);
        // Only the textured theme (added first, so index 0) contributes a UV set.
        assert_eq!(app.themes()[0].theme, theme("rgb"));
        assert_eq!(app.themes()[0].uv_sets.len(), 1);
        assert!(app.themes()[1].uv_sets.is_empty());
    }

    #[test]
    fn two_sided_theme_binds_both_faces() {
        let mut p = triangle();
        let face = || PolygonFace::single(textured(), Some(uv(3)));
        p.set_two_sided_appearance(theme("rgb"), face(), face())
            .unwrap();

        let app = p.appearance().as_ref().unwrap();
        assert_eq!(app.materials().len(), 2);
        assert!(app.themes()[0].back.is_some());
        assert_eq!(app.themes()[0].uv_sets.len(), 2);
        let sides: Vec<Side> = app.themes()[0].uv_sets.iter().map(|u| u.side).collect();
        assert!(sides.contains(&Side::Front) && sides.contains(&Side::Back));
    }
}
