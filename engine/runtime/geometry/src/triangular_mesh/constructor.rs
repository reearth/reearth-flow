//! TriangularMesh constructors.
//!
//! Two entry points, matching how a triangle mesh's buffers actually arrive:
//!
//! * `from_parts` — the vertex pool is already in hand (glTF / OBJ accessors,
//!   earcut output over an input ring, any algorithm that tracks its own
//!   indices). The index width is fixed by the vertex count — the largest index a
//!   valid mesh can hold is `vertices.len() - 1` — so it is computed once, with no
//!   scan of the index values, and the triangles are packed straight into that
//!   exact width. Indices arrive as a flat `u32` stream (the glTF / earcut shape)
//!   and are grouped into triples; the checked form validates that every index is
//!   in range and the count is a multiple of three.
//!
//! * `from_soup` — a flat stream of triangle-corner coordinates with no sharing
//!   (STL, marching-cubes-style emitters). The vertex pool is *discovered* by
//!   deduplicating corners, so the final count — hence the index width — is not
//!   known up front; the indices grow through [`IndexBuffer::from_indices`], which
//!   starts at `u8` and widens only as the pool crosses a width boundary, keeping
//!   the common small mesh narrow.
//!
//! 3D constructors build the coordinate-free [`TriangularMesh3DData`] that a
//! [`Solid`](crate::solid::Solid) shell also stores, so the frame-carrying
//! [`TriangularMesh3D`] is a thin wrapper. All meshes are built bare (no UV, no
//! appearance); attach an appearance afterwards via `appearance_mut`.

use std::collections::HashMap;

use crate::appearance::{
    validate_uv_coupling, Appearance, FaceBinding, Material, MaterialIndex, Side, ThemeBinding,
    ThemeId, UvSet, UvSource,
};
use crate::coordinate::Coordinate;
use crate::error::Error;
use crate::index::{IndexBuffer, IndexWidth};

use super::{TriangularMesh2D, TriangularMesh3D, TriangularMesh3DData};

impl TriangularMesh3DData {
    /// Build coordinate-free mesh data from a vertex pool and a flat `u32` index
    /// stream. Validates that the index count is a multiple of three and every
    /// index is `< vertices.len()`; the width is taken from the vertex count.
    pub fn from_parts(
        vertices: Vec<[f64; 3]>,
        indices: impl IntoIterator<Item = u32>,
    ) -> Result<Self, Error> {
        let width = index_width_for(vertices.len());
        let triangles = triangles_checked(indices, vertices.len())?;
        Ok(Self {
            vertices,
            indices: pack_checked(width, triangles),
            uv_sets: Vec::new(),
            appearance: None,
        })
    }

    /// Build mesh data without validating indices, filling the index buffer by the
    /// fast uninitialized path.
    ///
    /// # Safety
    /// The caller must ensure every index is `< vertices.len()` (an out-of-range
    /// index is silently truncated into the vertex-count-derived width) and that
    /// the flat `indices` stream yields exactly `3 * triangle_count` items.
    #[allow(unused)] // TODO: remove this after the migration is complete at which point this will be used.
    pub(crate) unsafe fn from_parts_unchecked(
        vertices: Vec<[f64; 3]>,
        triangle_count: usize,
        indices: impl IntoIterator<Item = u32>,
    ) -> Self {
        let width = index_width_for(vertices.len());
        Self {
            indices: IndexBuffer::from_exact_unchecked(
                width,
                triangle_count,
                group_triples(indices),
            ),
            vertices,
            uv_sets: Vec::new(),
            appearance: None,
        }
    }

    /// Build mesh data from a triangle soup: a flat stream of corner coordinates,
    /// three per triangle, deduplicated into a shared vertex pool.
    pub fn from_soup(iter: impl IntoIterator<Item = [f64; 3]>) -> Self {
        let (vertices, indices) = soup_buffers::<3>(iter);
        Self {
            vertices,
            indices,
            uv_sets: Vec::new(),
            appearance: None,
        }
    }

    /// Add a single-material, front-side appearance for one theme — a `Uniform`
    /// binding, so the whole mesh uses `material` under that theme. **Additive**:
    /// call once per theme to build a multi-theme mesh; the first theme added
    /// becomes the default. `uv` is required iff `material` is textured (an
    /// `Explicit` array must have `3 * triangle_count` entries — one per corner),
    /// and forbidden otherwise. Errors (leaving the mesh unchanged) on a duplicate
    /// theme or a material / UV / length violation.
    pub fn set_appearance(
        &mut self,
        theme: ThemeId,
        material: Material,
        uv: Option<UvSource>,
    ) -> Result<(), Error> {
        let binding = FaceBinding::Uniform(MaterialIndex::new(0).expect("0 is not u32::MAX"));
        self.set_appearance_with_binding(theme, vec![material], binding, uv)
    }

    /// Add a multi-material, front-side appearance for one theme with an explicit
    /// per-triangle `binding`. `binding` indexes `materials` locally
    /// (`0..materials.len()`); those indices are offset into the accumulated
    /// palette. A `PerFace` binding's length must equal the triangle count.
    /// Additive and validated like [`set_appearance`](Self::set_appearance).
    pub fn set_appearance_with_binding(
        &mut self,
        theme: ThemeId,
        materials: Vec<Material>,
        binding: FaceBinding,
        uv: Option<UvSource>,
    ) -> Result<(), Error> {
        add_theme(
            self.indices.len(),
            &mut self.appearance,
            &mut self.uv_sets,
            theme,
            materials,
            binding,
            uv,
        )
    }
}

impl TriangularMesh3D {
    /// Pair coordinate-free mesh data with the frame it is expressed in.
    pub fn new(coordinate: Coordinate, data: TriangularMesh3DData) -> Self {
        Self { coordinate, data }
    }

    /// Build from a vertex pool and a flat `u32` index stream; see
    /// [`TriangularMesh3DData::from_parts`].
    pub fn from_parts(
        coordinate: Coordinate,
        vertices: Vec<[f64; 3]>,
        indices: impl IntoIterator<Item = u32>,
    ) -> Result<Self, Error> {
        Ok(Self::new(
            coordinate,
            TriangularMesh3DData::from_parts(vertices, indices)?,
        ))
    }

    /// Build without validating indices; see
    /// [`TriangularMesh3DData::from_parts_unchecked`].
    ///
    /// # Safety
    /// Same contract as [`TriangularMesh3DData::from_parts_unchecked`].
    #[allow(unused)] // TODO: remove this after the migration is complete at which point this will be used.
    pub unsafe fn from_parts_unchecked(
        coordinate: Coordinate,
        vertices: Vec<[f64; 3]>,
        triangle_count: usize,
        indices: impl IntoIterator<Item = u32>,
    ) -> Self {
        Self::new(
            coordinate,
            TriangularMesh3DData::from_parts_unchecked(vertices, triangle_count, indices),
        )
    }

    /// Build from a triangle soup; see [`TriangularMesh3DData::from_soup`].
    pub fn from_soup(coordinate: Coordinate, iter: impl IntoIterator<Item = [f64; 3]>) -> Self {
        Self::new(coordinate, TriangularMesh3DData::from_soup(iter))
    }

    /// Add a single-material appearance for one theme; see
    /// [`TriangularMesh3DData::set_appearance`].
    pub fn set_appearance(
        &mut self,
        theme: ThemeId,
        material: Material,
        uv: Option<UvSource>,
    ) -> Result<(), Error> {
        self.data.set_appearance(theme, material, uv)
    }

    /// Add a multi-material appearance for one theme; see
    /// [`TriangularMesh3DData::set_appearance_with_binding`].
    pub fn set_appearance_with_binding(
        &mut self,
        theme: ThemeId,
        materials: Vec<Material>,
        binding: FaceBinding,
        uv: Option<UvSource>,
    ) -> Result<(), Error> {
        self.data
            .set_appearance_with_binding(theme, materials, binding, uv)
    }
}

impl TriangularMesh2D {
    /// Build a pure-2D mesh from a vertex pool and a flat `u32` index stream.
    /// Validates the index count and range; the width is taken from the vertex
    /// count.
    pub fn from_parts(
        coordinate: Coordinate,
        vertices: Vec<[f64; 2]>,
        indices: impl IntoIterator<Item = u32>,
    ) -> Result<Self, Error> {
        let width = index_width_for(vertices.len());
        let triangles = triangles_checked(indices, vertices.len())?;
        Ok(Self {
            coordinate,
            vertices,
            z: None,
            indices: pack_checked(width, triangles),
            uv_sets: Vec::new(),
            appearance: None,
        })
    }

    /// Build a 2.5D mesh from `[x, y, z]` vertices: the `(x, y)` populate the
    /// vertex pool and the `z` the parallel elevation buffer.
    pub fn from_parts_with_elevation(
        coordinate: Coordinate,
        vertices: Vec<[f64; 3]>,
        indices: impl IntoIterator<Item = u32>,
    ) -> Result<Self, Error> {
        let width = index_width_for(vertices.len());
        let triangles = triangles_checked(indices, vertices.len())?;
        let (xy, z) = split_elevation(vertices);
        Ok(Self {
            coordinate,
            vertices: xy,
            z: Some(z),
            indices: pack_checked(width, triangles),
            uv_sets: Vec::new(),
            appearance: None,
        })
    }

    /// Build a pure-2D mesh without validating indices.
    ///
    /// # Safety
    /// Same contract as [`TriangularMesh3DData::from_parts_unchecked`].
    pub unsafe fn from_parts_unchecked(
        coordinate: Coordinate,
        vertices: Vec<[f64; 2]>,
        triangle_count: usize,
        indices: impl IntoIterator<Item = u32>,
    ) -> Self {
        let width = index_width_for(vertices.len());
        Self {
            coordinate,
            indices: IndexBuffer::from_exact_unchecked(
                width,
                triangle_count,
                group_triples(indices),
            ),
            vertices,
            z: None,
            uv_sets: Vec::new(),
            appearance: None,
        }
    }

    /// Build a pure-2D mesh from a triangle soup of `[x, y]` corners.
    pub fn from_soup(coordinate: Coordinate, iter: impl IntoIterator<Item = [f64; 2]>) -> Self {
        let (vertices, indices) = soup_buffers::<2>(iter);
        Self {
            coordinate,
            vertices,
            z: None,
            indices,
            uv_sets: Vec::new(),
            appearance: None,
        }
    }

    /// Add a single-material appearance for one theme; see
    /// [`TriangularMesh3DData::set_appearance`].
    pub fn set_appearance(
        &mut self,
        theme: ThemeId,
        material: Material,
        uv: Option<UvSource>,
    ) -> Result<(), Error> {
        let binding = FaceBinding::Uniform(MaterialIndex::new(0).expect("0 is not u32::MAX"));
        self.set_appearance_with_binding(theme, vec![material], binding, uv)
    }

    /// Add a multi-material appearance for one theme; see
    /// [`TriangularMesh3DData::set_appearance_with_binding`].
    pub fn set_appearance_with_binding(
        &mut self,
        theme: ThemeId,
        materials: Vec<Material>,
        binding: FaceBinding,
        uv: Option<UvSource>,
    ) -> Result<(), Error> {
        add_theme(
            self.indices.len(),
            &mut self.appearance,
            &mut self.uv_sets,
            theme,
            materials,
            binding,
            uv,
        )
    }
}

// ─── Appearance ──────────────────────────────────────────────────────────────
//
// One theme is added per call (shared by the 2D and 3D setters). UV is per-corner
// — `3 * triangle_count` entries — and the binding is genuinely multi-face, so a
// `PerFace` binding carries one material per triangle.

/// Add one theme's appearance to a triangle mesh's `appearance` / `uv_sets`.
/// `binding` indexes `materials` locally; its indices are offset into the
/// accumulated palette. Validates the binding shape, the material/UV coupling and
/// (for `Explicit`) the UV length against `3 * triangle_count`. On any error the
/// mesh is left unchanged; on success the first theme added becomes the default.
fn add_theme(
    triangle_count: usize,
    appearance: &mut Option<Appearance>,
    uv_sets: &mut Vec<UvSet>,
    theme: ThemeId,
    materials: Vec<Material>,
    binding: FaceBinding,
    uv: Option<UvSource>,
) -> Result<(), Error> {
    if let Some(app) = appearance.as_ref() {
        if app.themes.iter().any(|b| b.theme == theme) {
            return Err(Error::invalid_appearance(format!(
                "theme `{}` is already set",
                theme.0
            )));
        }
    }
    validate_binding(&binding, materials.len(), triangle_count)?;
    validate_uv_coupling(
        references_texture(&binding, &materials),
        &uv,
        triangle_count * 3,
    )?;

    let app = appearance.get_or_insert_with(|| Appearance {
        materials: Vec::new(),
        themes: Vec::new(),
        default_theme: theme.clone(),
    });
    let offset = app.materials.len() as u32;
    app.materials.extend(materials);
    let front = offset_binding(binding, offset)?;
    app.themes.push(ThemeBinding {
        theme: theme.clone(),
        front,
        back: None,
    });
    if let Some(uv) = uv {
        uv_sets.push(UvSet {
            theme: Some(theme),
            side: Side::Front,
            channel: None,
            uv,
        });
    }
    Ok(())
}

/// Check a binding references only `0..palette_len` and, when `PerFace`, has one
/// entry per triangle.
fn validate_binding(
    binding: &FaceBinding,
    palette_len: usize,
    triangle_count: usize,
) -> Result<(), Error> {
    let in_range = |index: MaterialIndex| (index.get() as usize) < palette_len;
    match binding {
        FaceBinding::Uniform(index) => {
            if !in_range(*index) {
                return Err(Error::invalid_appearance(format!(
                    "material index {} is out of range for {palette_len} materials",
                    index.get()
                )));
            }
        }
        FaceBinding::PerFace(faces) => {
            if faces.len() != triangle_count {
                return Err(Error::invalid_appearance(format!(
                    "per-face binding length {} does not match the triangle count {triangle_count}",
                    faces.len()
                )));
            }
            if faces.iter().flatten().any(|index| !in_range(*index)) {
                return Err(Error::invalid_appearance(format!(
                    "a per-face material index is out of range for {palette_len} materials"
                )));
            }
        }
    }
    Ok(())
}

/// Whether any material the binding references carries a texture (and so needs UV).
fn references_texture(binding: &FaceBinding, materials: &[Material]) -> bool {
    let textured = |index: MaterialIndex| {
        materials
            .get(index.get() as usize)
            .is_some_and(Material::has_texture)
    };
    match binding {
        FaceBinding::Uniform(index) => textured(*index),
        FaceBinding::PerFace(faces) => faces.iter().flatten().copied().any(textured),
    }
}

/// Shift a binding's local material indices into the merged palette by `offset`.
fn offset_binding(binding: FaceBinding, offset: u32) -> Result<FaceBinding, Error> {
    let shift = |index: MaterialIndex| {
        MaterialIndex::new(index.get() + offset)
            .ok_or_else(|| Error::invalid_appearance("material palette too large"))
    };
    Ok(match binding {
        FaceBinding::Uniform(index) => FaceBinding::Uniform(shift(index)?),
        FaceBinding::PerFace(faces) => FaceBinding::PerFace(
            faces
                .into_iter()
                .map(|opt| opt.map(shift).transpose())
                .collect::<Result<_, _>>()?,
        ),
    })
}

/// Narrowest index width that can address `vertex_count` vertices (largest index
/// `vertex_count - 1`); `IndexWidth::U8` for the empty mesh.
fn index_width_for(vertex_count: usize) -> IndexWidth {
    IndexWidth::for_value(vertex_count.saturating_sub(1) as u32)
}

/// Group a flat index stream into triangles, validating that the count is a
/// multiple of three and every index addresses a vertex `< vertex_count`.
fn triangles_checked(
    indices: impl IntoIterator<Item = u32>,
    vertex_count: usize,
) -> Result<Vec<[u32; 3]>, Error> {
    let mut it = indices.into_iter();
    let mut triangles = Vec::with_capacity(it.size_hint().0 / 3);
    while let Some(a) = it.next() {
        let (Some(b), Some(c)) = (it.next(), it.next()) else {
            return Err(Error::invalid_geometry(
                "triangle index count is not a multiple of three",
            ));
        };
        for index in [a, b, c] {
            if index as usize >= vertex_count {
                return Err(Error::invalid_geometry(format!(
                    "triangle index {index} is out of range for {vertex_count} vertices"
                )));
            }
        }
        triangles.push([a, b, c]);
    }
    Ok(triangles)
}

/// Group a flat index stream into triangles, dropping a trailing partial triple.
/// No validation — for the unchecked path only.
fn group_triples(indices: impl IntoIterator<Item = u32>) -> impl Iterator<Item = [u32; 3]> {
    let mut it = indices.into_iter();
    std::iter::from_fn(move || Some([it.next()?, it.next()?, it.next()?]))
}

/// Pack already-range-checked triangles into the vertex-count-derived `width`.
/// Every index was validated `< vertex_count`, so it fits `width` (derived from
/// `vertex_count - 1`) and `with_exact_width` never panics here.
fn pack_checked(width: IndexWidth, triangles: Vec<[u32; 3]>) -> IndexBuffer<3> {
    IndexBuffer::with_exact_width(width, Some(triangles.len()), triangles)
}

/// Index of `coord` in `vertices`, inserting it (and assigning the next index) on
/// first sight. Dedup is on exact `f64` bits.
fn dedup_index<const N: usize>(
    vertices: &mut Vec<[f64; N]>,
    seen: &mut HashMap<[u64; N], u32>,
    coord: [f64; N],
) -> u32 {
    *seen.entry(coord.map(f64::to_bits)).or_insert_with(|| {
        let index = vertices.len() as u32;
        vertices.push(coord);
        index
    })
}

/// Deduplicate a flat triangle-corner stream into a vertex pool and a grown index
/// buffer, shared by the 2D and 3D soup constructors. A trailing partial triangle
/// is dropped.
fn soup_buffers<const N: usize>(
    iter: impl IntoIterator<Item = [f64; N]>,
) -> (Vec<[f64; N]>, IndexBuffer<3>) {
    let mut vertices: Vec<[f64; N]> = Vec::new();
    let mut seen: HashMap<[u64; N], u32> = HashMap::new();
    let mut src = iter.into_iter();
    let vertices_ref = &mut vertices;
    let seen_ref = &mut seen;
    let triangles = std::iter::from_fn(move || {
        let a = src.next()?;
        let b = src.next()?;
        let c = src.next()?;
        Some([
            dedup_index(vertices_ref, seen_ref, a),
            dedup_index(vertices_ref, seen_ref, b),
            dedup_index(vertices_ref, seen_ref, c),
        ])
    });
    let indices = IndexBuffer::from_indices(triangles);
    (vertices, indices)
}

/// Split a 3D vertex buffer into its 2D footprint and a parallel elevation buffer.
fn split_elevation(vertices: Vec<[f64; 3]>) -> (Vec<[f64; 2]>, Box<[f64]>) {
    let mut xy = Vec::with_capacity(vertices.len());
    let mut z = Vec::with_capacity(vertices.len());
    for [x, y, elevation] in vertices {
        xy.push([x, y]);
        z.push(elevation);
    }
    (xy, z.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    fn triples(buf: &IndexBuffer<3>) -> Vec<[u32; 3]> {
        match buf {
            IndexBuffer::U8(v) => v.iter().map(|t| (*t).map(u32::from)).collect(),
            IndexBuffer::U16(v) => v.iter().map(|t| (*t).map(u32::from)).collect(),
            IndexBuffer::U32(v) => v.iter().map(|t| (*t).map(u32::from)).collect(),
        }
    }

    #[test]
    fn from_parts_builds_validated_mesh() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let m = TriangularMesh3D::from_parts(Coordinate::Euclidean, verts, [0u32, 1, 2]).unwrap();
        assert_eq!(m.data.vertices.len(), 3);
        assert_eq!(m.data.indices.width(), IndexWidth::U8);
        assert_eq!(triples(&m.data.indices), vec![[0, 1, 2]]);
    }

    #[test]
    fn from_parts_width_follows_vertex_count_not_indices() {
        // 300 vertices -> u16, even though the one triangle uses only small indices.
        let verts: Vec<[f64; 3]> = (0..300).map(|i| [i as f64, 0.0, 0.0]).collect();
        let m = TriangularMesh3DData::from_parts(verts, [0u32, 1, 2]).unwrap();
        assert_eq!(m.indices.width(), IndexWidth::U16);
    }

    #[test]
    fn from_parts_rejects_out_of_range_index() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let err = TriangularMesh3DData::from_parts(verts, [0u32, 1, 3]).unwrap_err();
        assert!(matches!(err, Error::InvalidGeometry(_)));
    }

    #[test]
    fn from_parts_rejects_non_multiple_of_three() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        assert!(TriangularMesh3DData::from_parts(verts, [0u32, 1]).is_err());
    }

    #[test]
    fn from_soup_dedups_shared_vertices() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let d = [1.0, 1.0, 0.0];
        // Two triangles sharing edge b-c: a b c, b c d -> 4 unique vertices.
        let m = TriangularMesh3DData::from_soup([a, b, c, b, c, d]);
        assert_eq!(m.vertices, vec![a, b, c, d]);
        assert_eq!(triples(&m.indices), vec![[0, 1, 2], [1, 2, 3]]);
        assert_eq!(m.indices.width(), IndexWidth::U8);
    }

    #[test]
    fn from_soup_drops_trailing_partial_triangle() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let d = [1.0, 1.0, 0.0];
        let e = [2.0, 2.0, 0.0];
        // One full triangle plus two leftover corners: d and e are never consumed
        // into a triple, so they are not added to the pool.
        let m = TriangularMesh3DData::from_soup([a, b, c, d, e]);
        assert_eq!(triples(&m.indices), vec![[0, 1, 2]]);
        assert_eq!(m.vertices, vec![a, b, c]);
    }

    #[test]
    fn from_parts_with_elevation_splits_z() {
        let verts = vec![[0.0, 0.0, 10.0], [1.0, 0.0, 11.0], [0.0, 1.0, 12.0]];
        let m =
            TriangularMesh2D::from_parts_with_elevation(Coordinate::Euclidean, verts, [0u32, 1, 2])
                .unwrap();
        assert_eq!(m.vertices, vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
        assert_eq!(m.z.as_deref(), Some(&[10.0, 11.0, 12.0][..]));
    }

    #[test]
    fn from_parts_unchecked_matches_checked() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let checked = TriangularMesh3DData::from_parts(verts.clone(), [0u32, 1, 2]).unwrap();
        let unchecked =
            unsafe { TriangularMesh3DData::from_parts_unchecked(verts, 1, [0u32, 1, 2]) };
        assert_eq!(checked, unchecked);
    }

    // ── Appearance setters ──

    /// One triangle (3 corners).
    fn one_triangle() -> TriangularMesh3DData {
        TriangularMesh3DData::from_parts(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap()
    }

    /// Two triangles (6 corners) over a quad.
    fn two_triangles() -> TriangularMesh3DData {
        TriangularMesh3DData::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            [0u32, 1, 2, 0, 2, 3],
        )
        .unwrap()
    }

    #[test]
    fn set_appearance_uniform_textured() {
        let mut m = one_triangle();
        m.set_appearance(theme("rgb"), textured(), Some(uv(3)))
            .unwrap();
        let app = m.appearance.as_ref().unwrap();
        assert_eq!(app.materials.len(), 1);
        assert_eq!(app.default_theme, theme("rgb"));
        assert!(matches!(app.themes[0].front, FaceBinding::Uniform(_)));
        assert!(app.themes[0].back.is_none());
        assert_eq!(m.uv_sets.len(), 1);
        assert_eq!(m.uv_sets[0].side, Side::Front);
    }

    #[test]
    fn set_appearance_uniform_bare_has_no_uv() {
        let mut m = one_triangle();
        m.set_appearance(theme("rgb"), bare(), None).unwrap();
        assert_eq!(m.appearance.as_ref().unwrap().materials.len(), 1);
        assert!(m.uv_sets.is_empty());
    }

    #[test]
    fn set_appearance_textured_without_uv_is_rejected() {
        let mut m = one_triangle();
        let err = m
            .set_appearance(theme("rgb"), textured(), None)
            .unwrap_err();
        assert!(matches!(err, Error::InvalidAppearance(_)));
        assert!(m.appearance.is_none(), "left unchanged");
    }

    #[test]
    fn set_appearance_uv_length_must_match_corner_count() {
        let mut m = one_triangle();
        // 3 corners expected, 4 supplied.
        let err = m
            .set_appearance(theme("rgb"), textured(), Some(uv(4)))
            .unwrap_err();
        assert!(matches!(err, Error::InvalidAppearance(_)));
    }

    #[test]
    fn set_appearance_with_per_face_binding() {
        let mut m = two_triangles();
        let binding = FaceBinding::PerFace(vec![MaterialIndex::new(0), MaterialIndex::new(1)]);
        m.set_appearance_with_binding(theme("rgb"), vec![textured(), bare()], binding, Some(uv(6)))
            .unwrap();
        let app = m.appearance.as_ref().unwrap();
        assert_eq!(app.materials.len(), 2);
        let FaceBinding::PerFace(faces) = &app.themes[0].front else {
            panic!("expected PerFace");
        };
        assert_eq!(faces, &[MaterialIndex::new(0), MaterialIndex::new(1)]);
        let UvSource::Explicit(coords) = &m.uv_sets[0].uv else {
            panic!("expected Explicit");
        };
        assert_eq!(coords.len(), 6);
    }

    #[test]
    fn per_face_binding_length_must_match_triangle_count() {
        let mut m = two_triangles(); // 2 triangles
        let binding = FaceBinding::PerFace(vec![MaterialIndex::new(0)]); // only 1 entry
        let err = m
            .set_appearance_with_binding(theme("rgb"), vec![textured()], binding, Some(uv(6)))
            .unwrap_err();
        assert!(matches!(err, Error::InvalidAppearance(_)));
    }

    #[test]
    fn binding_index_out_of_range_is_rejected() {
        let mut m = one_triangle();
        let binding = FaceBinding::Uniform(MaterialIndex::new(5).unwrap()); // no such material
        let err = m
            .set_appearance_with_binding(theme("rgb"), vec![textured()], binding, Some(uv(3)))
            .unwrap_err();
        assert!(matches!(err, Error::InvalidAppearance(_)));
    }

    #[test]
    fn themes_accumulate_one_at_a_time() {
        let mut m = two_triangles();
        m.set_appearance(theme("rgb"), textured(), Some(uv(6)))
            .unwrap();
        m.set_appearance(theme("infrared"), bare(), None).unwrap();

        let app = m.appearance.as_ref().unwrap();
        assert_eq!(app.themes.len(), 2);
        assert_eq!(
            app.default_theme,
            theme("rgb"),
            "first theme is the default"
        );
        assert_eq!(app.materials.len(), 2);
        // The second theme's local index 0 is offset to palette slot 1.
        let FaceBinding::Uniform(rgb) = app.themes[0].front else {
            panic!();
        };
        let FaceBinding::Uniform(infrared) = app.themes[1].front else {
            panic!();
        };
        assert_eq!(rgb.get(), 0);
        assert_eq!(infrared.get(), 1);
        // Only the textured theme contributes a UV set.
        assert_eq!(m.uv_sets.len(), 1);
        assert_eq!(m.uv_sets[0].theme.as_ref(), Some(&theme("rgb")));
    }

    #[test]
    fn duplicate_theme_is_rejected() {
        let mut m = one_triangle();
        m.set_appearance(theme("rgb"), bare(), None).unwrap();
        let err = m.set_appearance(theme("rgb"), bare(), None).unwrap_err();
        assert!(matches!(err, Error::InvalidAppearance(_)));
    }
}
