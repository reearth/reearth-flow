//! PolygonMesh constructors.
//!
//! Three entry points, matching how a polygon mesh's faces arrive:
//!
//! * `from_polygons` — a set of independent [`Polygon`](crate::polygon) faces, the
//!   shape a CityGML surface parses into (a flat list of face polygons, each with
//!   its own coords and holes, no shared pool). The shared vertex pool is
//!   *discovered* by deduplicating face corners (exact `f64` bits), and each
//!   polygon maps 1:1 to a mesh face — exterior then holes. Because the CityGML
//!   reader also emits each face as its own `Polygon` feature, the mesh is built
//!   from those same borrowed polygons with no extra geometry logic.
//!
//! * `from_parts` — a vertex pool already in hand plus faces given as vertex-index
//!   lists (OBJ). Exterior-only (standard OBJ has no holes); indices are validated
//!   `< vertices.len()`.
//!
//! * `from_raw_parts` — the flat CSR buffers in hand, validated for consistency.
//!
//! Rings are stored **open**: the closing vertex is implied by the face / hole
//! boundary, so `from_polygons` drops the closing duplicate that a `Polygon`
//! carries, and `from_parts` expects the open faces OBJ provides. The index widths
//! follow the type's contract: `face_indices` from the (dedup-discovered) vertex
//! count, `face_offsets` / `interior_offsets` from `face_indices.len()`.
//!
//! 3D constructors build the coordinate-free [`PolygonMesh3DData`] a
//! [`Solid`](crate::solid::Solid) shell also stores, so the frame-carrying
//! [`PolygonMesh3D`] is a thin wrapper. `from_parts` / `from_raw_parts` build
//! bare (attach appearance later via `appearance_mut`); `from_polygons` welds
//! the constituent polygons' appearance and UV across into the mesh.

use std::collections::{HashMap, HashSet};

use crate::appearance::{
    Appearance, ChannelId, FaceBinding, Material, MaterialIndex, Side, TexMatrix, ThemeBinding,
    ThemeId, UvSet, UvSource,
};
use crate::coordinate::Coordinate;
use crate::error::Error;
use crate::index::{IndexBuffer, IndexWidth};
use crate::polygon::Polygon3D;

use super::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};

impl PolygonMesh3DData {
    /// Build coordinate-free mesh data from independent face polygons, deduplicating
    /// their corners into a shared vertex pool. Each polygon becomes one face
    /// (exterior then holes), stored open.
    ///
    /// Appearance and UV are merged across the faces (see [`merge_appearance`]):
    /// material palettes concatenate, themes union, each face gets a `PerFace`
    /// binding, and every per-face UV is welded into one mesh-wide array per
    /// `(theme, side, channel)` — dropping each ring's closing duplicate to match
    /// the open corner buffer, baking any `WorldToTexture` matrix to explicit
    /// corners, and zero-filling faces a theme does not cover (their binding is
    /// `None`, so the filler is never sampled). Bare input (no appearance on any
    /// polygon) yields a bare mesh.
    ///
    /// A welded multi-face mesh's faces carry *different* `WorldToTexture` matrices
    /// and cannot share one, so any matrix UV is baked to per-corner `Explicit` here
    /// (a single-surface triangulation keeps its one matrix instead — see
    /// [`UvSource`]).
    pub fn from_polygons<'a>(polygons: impl IntoIterator<Item = &'a Polygon3D>) -> Self {
        Self::from_polygons_with_default_theme(polygons, None)
    }

    /// As [`from_polygons`](Self::from_polygons), but pins the merged mesh's active
    /// default theme. `default_theme` should name one of the welded themes; `None`
    /// keeps the auto rule (the shared default when every appearance-carrying face
    /// agrees on one, otherwise the first kept face's default).
    pub fn from_polygons_with_default_theme<'a>(
        polygons: impl IntoIterator<Item = &'a Polygon3D>,
        default_theme: Option<ThemeId>,
    ) -> Self {
        let polygons: Vec<&Polygon3D> = polygons.into_iter().collect();
        let faces = polygons
            .iter()
            .map(|p| std::iter::once(p.exterior()).chain(p.interiors()));
        let (vertices, face_indices, face_offsets, interior_offsets) = dedup_faces(faces);
        let (uv_sets, appearance) = merge_appearance(&polygons, default_theme);
        let (face_indices, face_offsets, interior_offsets) =
            pack_csr(vertices.len(), face_indices, face_offsets, interior_offsets);
        Self {
            vertices,
            face_indices,
            face_offsets,
            interior_offsets,
            uv_sets,
            appearance,
        }
    }

    /// Build from a shared vertex pool and faces given as vertex-index lists
    /// (exterior rings, no holes). Validates every index is `< vertices.len()`.
    pub fn from_parts(
        vertices: Vec<[f64; 3]>,
        faces: impl IntoIterator<Item = impl IntoIterator<Item = u32>>,
    ) -> Result<Self, Error> {
        let (face_indices, face_offsets) = index_faces(vertices.len(), faces)?;
        let (face_indices, face_offsets, interior_offsets) =
            pack_csr(vertices.len(), face_indices, face_offsets, Vec::new());
        Ok(Self {
            vertices,
            face_indices,
            face_offsets,
            interior_offsets,
            uv_sets: Vec::new(),
            appearance: None,
        })
    }

    /// Build from flat CSR buffers, validating that the offsets are consistent and
    /// every index is in range.
    pub fn from_raw_parts(
        vertices: Vec<[f64; 3]>,
        face_indices: Vec<u32>,
        face_offsets: Vec<u32>,
        interior_offsets: Vec<u32>,
    ) -> Result<Self, Error> {
        validate_csr(
            vertices.len(),
            &face_indices,
            &face_offsets,
            &interior_offsets,
        )?;
        let (face_indices, face_offsets, interior_offsets) =
            pack_csr(vertices.len(), face_indices, face_offsets, interior_offsets);
        Ok(Self {
            vertices,
            face_indices,
            face_offsets,
            interior_offsets,
            uv_sets: Vec::new(),
            appearance: None,
        })
    }
}

impl PolygonMesh3D {
    /// Pair coordinate-free mesh data with the frame it is expressed in.
    pub fn new(coordinate: Coordinate, data: PolygonMesh3DData) -> Self {
        Self { coordinate, data }
    }

    /// Build from independent face polygons; see [`PolygonMesh3DData::from_polygons`].
    /// Errors if any polygon's frame differs from `coordinate`.
    pub fn from_polygons<'a>(
        coordinate: Coordinate,
        polygons: impl IntoIterator<Item = &'a Polygon3D>,
    ) -> Result<Self, Error> {
        Self::from_polygons_with_default_theme(coordinate, polygons, None)
    }

    /// As [`from_polygons`](Self::from_polygons), but pins the merged mesh's active
    /// default theme; see
    /// [`PolygonMesh3DData::from_polygons_with_default_theme`].
    pub fn from_polygons_with_default_theme<'a>(
        coordinate: Coordinate,
        polygons: impl IntoIterator<Item = &'a Polygon3D>,
        default_theme: Option<ThemeId>,
    ) -> Result<Self, Error> {
        let polygons: Vec<&Polygon3D> = polygons.into_iter().collect();
        for p in &polygons {
            if p.coordinate() != &coordinate {
                return Err(Error::invalid_geometry(
                    "polygon coordinate frame differs from the mesh frame",
                ));
            }
        }
        Ok(Self::new(
            coordinate,
            PolygonMesh3DData::from_polygons_with_default_theme(polygons, default_theme),
        ))
    }

    /// Build from a vertex pool and index-list faces; see
    /// [`PolygonMesh3DData::from_parts`].
    pub fn from_parts(
        coordinate: Coordinate,
        vertices: Vec<[f64; 3]>,
        faces: impl IntoIterator<Item = impl IntoIterator<Item = u32>>,
    ) -> Result<Self, Error> {
        Ok(Self::new(
            coordinate,
            PolygonMesh3DData::from_parts(vertices, faces)?,
        ))
    }

    /// Build from flat CSR buffers; see [`PolygonMesh3DData::from_raw_parts`].
    pub fn from_raw_parts(
        coordinate: Coordinate,
        vertices: Vec<[f64; 3]>,
        face_indices: Vec<u32>,
        face_offsets: Vec<u32>,
        interior_offsets: Vec<u32>,
    ) -> Result<Self, Error> {
        Ok(Self::new(
            coordinate,
            PolygonMesh3DData::from_raw_parts(
                vertices,
                face_indices,
                face_offsets,
                interior_offsets,
            )?,
        ))
    }
}

impl PolygonMesh2D {
    /// Build a pure-2D mesh from a vertex pool and index-list faces (no holes).
    pub fn from_parts(
        coordinate: Coordinate,
        vertices: Vec<[f64; 2]>,
        faces: impl IntoIterator<Item = impl IntoIterator<Item = u32>>,
    ) -> Result<Self, Error> {
        let (face_indices, face_offsets) = index_faces(vertices.len(), faces)?;
        let (face_indices, face_offsets, interior_offsets) =
            pack_csr(vertices.len(), face_indices, face_offsets, Vec::new());
        Ok(Self {
            coordinate,
            vertices,
            z: None,
            face_indices,
            face_offsets,
            interior_offsets,
            uv_sets: Vec::new(),
            appearance: None,
        })
    }

    /// Build a 2.5D mesh from `[x, y, z]` vertices (the `(x, y)` populate the pool,
    /// the `z` a parallel elevation buffer) and index-list faces (no holes).
    pub fn from_parts_with_elevation(
        coordinate: Coordinate,
        vertices: Vec<[f64; 3]>,
        faces: impl IntoIterator<Item = impl IntoIterator<Item = u32>>,
    ) -> Result<Self, Error> {
        let (face_indices, face_offsets) = index_faces(vertices.len(), faces)?;
        // Split the `[x, y, z]` vertices into the 2D pool and a parallel elevation buffer.
        let mut xy = Vec::with_capacity(vertices.len());
        let mut z = Vec::with_capacity(vertices.len());
        for [x, y, elevation] in vertices {
            xy.push([x, y]);
            z.push(elevation);
        }
        let z = z.into_boxed_slice();
        let (face_indices, face_offsets, interior_offsets) =
            pack_csr(xy.len(), face_indices, face_offsets, Vec::new());
        Ok(Self {
            coordinate,
            vertices: xy,
            z: Some(z),
            face_indices,
            face_offsets,
            interior_offsets,
            uv_sets: Vec::new(),
            appearance: None,
        })
    }

    /// Build a pure-2D mesh from flat CSR buffers.
    pub fn from_raw_parts(
        coordinate: Coordinate,
        vertices: Vec<[f64; 2]>,
        face_indices: Vec<u32>,
        face_offsets: Vec<u32>,
        interior_offsets: Vec<u32>,
    ) -> Result<Self, Error> {
        validate_csr(
            vertices.len(),
            &face_indices,
            &face_offsets,
            &interior_offsets,
        )?;
        let (face_indices, face_offsets, interior_offsets) =
            pack_csr(vertices.len(), face_indices, face_offsets, interior_offsets);
        Ok(Self {
            coordinate,
            vertices,
            z: None,
            face_indices,
            face_offsets,
            interior_offsets,
            uv_sets: Vec::new(),
            appearance: None,
        })
    }
}

/// Drop a ring's closing vertex when it duplicates the first, yielding the open
/// ring (closure is implied by the face / hole boundary in the mesh).
fn open_ring<const N: usize>(ring: &[[f64; N]]) -> &[[f64; N]] {
    if ring.len() > 1 && ring.first() == ring.last() {
        &ring[..ring.len() - 1]
    } else {
        ring
    }
}

/// Deduplicate the corners of a sequence of faces (each a sequence of rings,
/// exterior first) into a shared pool and the flat CSR buffers.
fn dedup_faces<'a, const N: usize, Faces, Rings>(
    faces: Faces,
) -> (Vec<[f64; N]>, Vec<u32>, Vec<u32>, Vec<u32>)
where
    Faces: IntoIterator<Item = Rings>,
    Rings: IntoIterator<Item = &'a [[f64; N]]>,
{
    let mut vertices: Vec<[f64; N]> = Vec::new();
    let mut seen: HashMap<[u64; N], u32> = HashMap::new();
    let mut face_indices: Vec<u32> = Vec::new();
    let mut face_offsets: Vec<u32> = Vec::new();
    let mut interior_offsets: Vec<u32> = Vec::new();
    for face in faces {
        let index_start = face_indices.len();
        let interior_start = interior_offsets.len();
        let mut is_exterior = true;
        for ring in face {
            if !is_exterior {
                interior_offsets.push(face_indices.len() as u32);
            }
            is_exterior = false;
            for &coord in open_ring(ring) {
                // Index of `coord` in the pool, inserting it on first sight; dedup
                // is on exact `f64` bits.
                let index = *seen.entry(coord.map(f64::to_bits)).or_insert_with(|| {
                    let next = vertices.len() as u32;
                    vertices.push(coord);
                    next
                });
                face_indices.push(index);
            }
        }
        if face_indices.len() == index_start {
            // Empty face contributes no geometry: drop it (and any hole offsets it
            // recorded) so `face_offsets` stays strictly increasing with no leading 0.
            interior_offsets.truncate(interior_start);
            continue;
        }
        face_offsets.push(face_indices.len() as u32);
    }
    // `face_offsets` holds only the internal boundaries — no leading 0, no trailing
    // total — so drop the last face's end (the implicit `face_indices.len()`).
    face_offsets.pop();
    (vertices, face_indices, face_offsets, interior_offsets)
}

/// Flatten index-list faces (exterior rings, no holes) into `face_indices` and
/// `face_offsets`, validating every index is `< vertex_count`.
fn index_faces(
    vertex_count: usize,
    faces: impl IntoIterator<Item = impl IntoIterator<Item = u32>>,
) -> Result<(Vec<u32>, Vec<u32>), Error> {
    let mut face_indices: Vec<u32> = Vec::new();
    let mut face_offsets: Vec<u32> = Vec::new();
    for face in faces {
        let index_start = face_indices.len();
        for index in face {
            if index as usize >= vertex_count {
                return Err(Error::invalid_geometry(format!(
                    "face index {index} is out of range for {vertex_count} vertices"
                )));
            }
            face_indices.push(index);
        }
        if face_indices.len() == index_start {
            // Empty face: contributes no ring, so record no boundary for it (keeps
            // `face_offsets` strictly increasing with no leading 0).
            continue;
        }
        face_offsets.push(face_indices.len() as u32);
    }
    // Keep only the internal boundaries (no leading 0, no trailing total).
    face_offsets.pop();
    Ok((face_indices, face_offsets))
}

/// Validate flat CSR buffers. `face_offsets` holds the `n_faces - 1` internal face
/// boundaries (no leading 0): strictly increasing, each in `1..face_indices.len()`
/// (every face non-empty). Every vertex index is `< vertex_count`. Each interior
/// offset is strictly increasing, in `1..face_indices.len()`, and not a face
/// boundary — so a hole starts strictly inside a face, after its exterior.
fn validate_csr(
    vertex_count: usize,
    face_indices: &[u32],
    face_offsets: &[u32],
    interior_offsets: &[u32],
) -> Result<(), Error> {
    for &index in face_indices {
        if index as usize >= vertex_count {
            return Err(Error::invalid_geometry(format!(
                "face index {index} is out of range for {vertex_count} vertices"
            )));
        }
    }
    let mut prev = 0u32;
    for (i, &boundary) in face_offsets.iter().enumerate() {
        if boundary <= prev || boundary as usize >= face_indices.len() {
            return Err(Error::invalid_geometry(format!(
                "face_offsets[{i}] = {boundary} must be strictly increasing and within 1..{}",
                face_indices.len()
            )));
        }
        prev = boundary;
    }
    let mut prev = 0u32;
    for (i, &offset) in interior_offsets.iter().enumerate() {
        if offset <= prev
            || offset as usize >= face_indices.len()
            || face_offsets.binary_search(&offset).is_ok()
        {
            return Err(Error::invalid_geometry(format!(
                "interior_offsets[{i}] = {offset} must be strictly increasing, within \
                 1..{}, and not coincide with a face boundary",
                face_indices.len()
            )));
        }
        prev = offset;
    }
    Ok(())
}

/// Pack the flat CSR buffers into width-erased index buffers: `face_indices` at the
/// vertex-count-derived width, the offsets at the `face_indices.len() - 1` width
/// (the largest possible offset, since the internal boundaries carry no trailing
/// total and every offset is `< face_indices.len()`).
fn pack_csr(
    vertex_count: usize,
    face_indices: Vec<u32>,
    face_offsets: Vec<u32>,
    interior_offsets: Vec<u32>,
) -> (IndexBuffer<1>, IndexBuffer<1>, IndexBuffer<1>) {
    let index_width = IndexWidth::for_value(vertex_count.saturating_sub(1) as u32);
    let offset_width = IndexWidth::for_value(face_indices.len().saturating_sub(1) as u32);
    (
        IndexBuffer::with_exact_width(
            index_width,
            Some(face_indices.len()),
            face_indices.into_iter().map(|i| [i]),
        ),
        IndexBuffer::with_exact_width(
            offset_width,
            Some(face_offsets.len()),
            face_offsets.into_iter().map(|o| [o]),
        ),
        IndexBuffer::with_exact_width(
            offset_width,
            Some(interior_offsets.len()),
            interior_offsets.into_iter().map(|o| [o]),
        ),
    )
}

/// The number of mesh corners a polygon contributes — its rings with each
/// closing duplicate dropped, matching `dedup_faces` / [`open_ring`]. A face
/// with zero corners is skipped by `dedup_faces`, so it is dropped here too.
fn corner_count(polygon: &Polygon3D) -> usize {
    std::iter::once(polygon.exterior())
        .chain(polygon.interiors())
        .map(|ring| open_ring(ring).len())
        .sum()
}

/// Append one polygon's UV in mesh-corner order for a single `UvSource` directly
/// onto `out`: rings concatenated (exterior then interiors), each closing
/// duplicate dropped. `Explicit` slices the parallel array; `WorldToTexture` bakes
/// the matrix at each corner vertex. Writes straight into the caller's reserved
/// buffer — no per-face temporary.
fn extend_polygon_uv(out: &mut Vec<[f64; 2]>, polygon: &Polygon3D, source: &UvSource) {
    let mut pos = 0usize;
    for ring in std::iter::once(polygon.exterior()).chain(polygon.interiors()) {
        let open = open_ring(ring);
        match source {
            UvSource::Explicit(uv) => out.extend_from_slice(&uv[pos..pos + open.len()]),
            UvSource::WorldToTexture(matrix) => {
                out.extend(open.iter().map(|&vertex| bake(matrix, vertex)));
            }
        }
        pos += ring.len();
    }
}

/// Bake a `WorldToTexture` projective matrix at a vertex: `(s, t) = (s'/q', t'/q')`.
///
/// A vertex on the matrix's vanishing line makes `q` zero (or the products
/// non-finite), which would otherwise bake `inf`/`NaN` UVs that corrupt texture
/// sampling and break downstream tile encoders. Such degenerate corners have no
/// meaningful UV, so they fall back to the texture origin.
fn bake(matrix: &TexMatrix, [x, y, z]: [f64; 3]) -> [f64; 2] {
    let apply = |row: [f64; 4]| row[0] * x + row[1] * y + row[2] * z + row[3];
    let s = apply(matrix.0[0]);
    let t = apply(matrix.0[1]);
    let q = apply(matrix.0[2]);
    let (u, v) = (s / q, t / q);
    if u.is_finite() && v.is_finite() {
        [u, v]
    } else {
        [0.0, 0.0]
    }
}

/// The single material a polygon's (single-face) binding resolves to, if bound.
fn single_face_index(binding: &FaceBinding) -> Option<u32> {
    match binding {
        FaceBinding::Uniform(index) => Some(index.get()),
        FaceBinding::PerFace(faces) => faces.first().copied().flatten().map(|index| index.get()),
    }
}

/// Whether a polygon binds a back-side material under `theme`.
fn has_back(polygon: &Polygon3D, theme: &ThemeId) -> bool {
    polygon
        .appearance()
        .as_ref()
        .and_then(|app| app.themes.iter().find(|binding| &binding.theme == theme))
        .is_some_and(|binding| binding.back.is_some())
}

/// Build a `PerFace` binding for one theme and side over the kept faces, remapping
/// each polygon's local material index into the merged palette via `offset`.
fn build_binding(
    kept: &[&Polygon3D],
    offset: &[usize],
    theme: &ThemeId,
    side: Side,
) -> FaceBinding {
    let per_face = kept
        .iter()
        .enumerate()
        .map(|(i, polygon)| -> Option<MaterialIndex> {
            let app = polygon.appearance().as_ref()?;
            let binding = app.themes.iter().find(|b| &b.theme == theme)?;
            let face = match side {
                Side::Front => &binding.front,
                Side::Back => binding.back.as_ref()?,
            };
            let local = single_face_index(face)?;
            // Overflow (only from an oversized palette) falls through to an unbound face.
            let merged = u32::try_from(offset[i]).ok()?.checked_add(local)?;
            MaterialIndex::new(merged)
        })
        .collect();
    FaceBinding::PerFace(per_face)
}

/// Weld the per-polygon appearance and UV of the kept faces into the mesh-wide
/// `(uv_sets, appearance)`. Returns `(empty, None)` if no face carries appearance.
///
/// `default_override` pins the merged mesh's active default theme; `None` keeps
/// the auto rule (the shared default when every appearance-carrying face agrees,
/// otherwise the first kept face's default).
fn merge_appearance(
    polygons: &[&Polygon3D],
    default_override: Option<ThemeId>,
) -> (Vec<UvSet>, Option<Appearance>) {
    let kept: Vec<&Polygon3D> = polygons
        .iter()
        .copied()
        .filter(|p| corner_count(p) > 0)
        .collect();

    if kept.iter().all(|p| p.appearance().is_none()) {
        return (Vec::new(), None);
    }

    // Concatenate the material palettes, recording each face's offset for index
    // remapping; a bare face adds nothing but keeps its slot.
    let mut materials: Vec<Material> = Vec::new();
    let mut offset: Vec<usize> = Vec::with_capacity(kept.len());
    for polygon in &kept {
        offset.push(materials.len());
        if let Some(app) = polygon.appearance() {
            materials.extend(app.materials.iter().cloned());
        }
    }

    // Theme union (first-seen) and the active default theme.
    let mut theme_order: Vec<ThemeId> = Vec::new();
    for polygon in &kept {
        if let Some(app) = polygon.appearance() {
            for binding in &app.themes {
                if !theme_order.contains(&binding.theme) {
                    theme_order.push(binding.theme.clone());
                }
            }
        }
    }
    // An explicit override wins; otherwise take the first kept face's default,
    // which equals the shared default when every face agrees on one (rule 1) and
    // is the documented fallback when they diverge (rule 2).
    let default_theme = default_override.unwrap_or_else(|| {
        kept.iter()
            .find_map(|p| p.appearance().as_ref().map(|app| app.default_theme.clone()))
            .expect("a kept face carries appearance")
    });

    let themes: Vec<ThemeBinding> = theme_order
        .iter()
        .map(|theme| {
            let front = build_binding(&kept, &offset, theme, Side::Front);
            let back = kept
                .iter()
                .any(|p| has_back(p, theme))
                .then(|| build_binding(&kept, &offset, theme, Side::Back));
            ThemeBinding {
                theme: theme.clone(),
                front,
                back,
            }
        })
        .collect();

    // Index each face's UV sets by key once (so the per-key build is a map lookup,
    // not a linear scan) and precompute each face's corner count (recomputing it
    // per key would re-walk every ring).
    type UvKey = (Option<ThemeId>, Side, ChannelId);
    let corner_counts: Vec<usize> = kept.iter().map(|p| corner_count(p)).collect();
    let total_corners: usize = corner_counts.iter().sum();
    let per_face_uv: Vec<HashMap<UvKey, &UvSource>> = kept
        .iter()
        .map(|polygon| {
            polygon
                .uv_sets()
                .iter()
                .map(|set| ((set.theme.clone(), set.side, set.channel), &set.uv))
                .collect()
        })
        .collect();

    // UV-set key union, in first-seen order.
    let mut keys: Vec<UvKey> = Vec::new();
    let mut seen: HashSet<UvKey> = HashSet::new();
    for polygon in &kept {
        for set in polygon.uv_sets() {
            let key = (set.theme.clone(), set.side, set.channel);
            if seen.insert(key.clone()) {
                keys.push(key);
            }
        }
    }

    // One mesh-wide UV array per key: each face contributes its corners, or a
    // zero-filled run where it lacks that key. Reserved to the total up front.
    let uv_sets = keys
        .into_iter()
        .map(|key| {
            let mut data: Vec<[f64; 2]> = Vec::with_capacity(total_corners);
            for (i, polygon) in kept.iter().enumerate() {
                match per_face_uv[i].get(&key).copied() {
                    Some(uv) => extend_polygon_uv(&mut data, polygon, uv),
                    None => data.resize(data.len() + corner_counts[i], [0.0, 0.0]),
                }
            }
            let (theme, side, channel) = key;
            UvSet {
                theme,
                side,
                channel,
                uv: UvSource::Explicit(data.into_boxed_slice()),
            }
        })
        .collect();

    (
        uv_sets,
        Some(Appearance {
            materials,
            themes,
            default_theme,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::EpsgCode;
    use crate::test_support::*;

    fn ones(buf: &IndexBuffer<1>) -> Vec<u32> {
        match buf {
            IndexBuffer::U8(v) => v.iter().map(|t| u32::from(t[0])).collect(),
            IndexBuffer::U16(v) => v.iter().map(|t| u32::from(t[0])).collect(),
            IndexBuffer::U32(v) => v.iter().map(|t| t[0]).collect(),
        }
    }

    fn quad(corners: [[f64; 3]; 4]) -> Polygon3D {
        Polygon3D::from_rings(Coordinate::Euclidean, corners, Vec::<Vec<[f64; 3]>>::new())
    }

    #[test]
    fn from_polygons_dedups_shared_edge() {
        // Two unit quads sharing the edge (1,0,0)-(1,1,0): 6 unique vertices.
        let a = quad([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        let b = quad([[1., 0., 0.], [2., 0., 0.], [2., 1., 0.], [1., 1., 0.]]);
        let m = PolygonMesh3D::from_polygons(Coordinate::Euclidean, [&a, &b]).unwrap();
        assert_eq!(
            m.data.vertices,
            vec![
                [0., 0., 0.],
                [1., 0., 0.],
                [1., 1., 0.],
                [0., 1., 0.],
                [2., 0., 0.],
                [2., 1., 0.],
            ]
        );
        // Rings stored open (the closing duplicate is dropped).
        assert_eq!(ones(&m.data.face_indices), vec![0, 1, 2, 3, 1, 4, 5, 2]);
        // Internal boundary only: face 0 ends / face 1 begins at 4.
        assert_eq!(ones(&m.data.face_offsets), vec![4]);
        assert!(ones(&m.data.interior_offsets).is_empty());
    }

    #[test]
    fn from_polygons_preserves_a_hole() {
        let outer = [[0., 0., 0.], [4., 0., 0.], [4., 4., 0.], [0., 4., 0.]];
        let hole = vec![[1., 1., 0.], [2., 1., 0.], [2., 2., 0.], [1., 2., 0.]];
        let p = Polygon3D::from_rings(Coordinate::Euclidean, outer, vec![hole]);
        let m = PolygonMesh3DData::from_polygons([&p]);
        assert_eq!(m.vertices.len(), 8);
        assert_eq!(ones(&m.face_indices), vec![0, 1, 2, 3, 4, 5, 6, 7]);
        // Single face -> no internal boundaries; the hole starts at index 4.
        assert!(ones(&m.face_offsets).is_empty());
        assert_eq!(ones(&m.interior_offsets), vec![4]);
    }

    #[test]
    fn from_polygons_rejects_frame_mismatch() {
        let a = quad([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        let err =
            PolygonMesh3D::from_polygons(Coordinate::Crs(EpsgCode::new(4326)), [&a]).unwrap_err();
        assert!(matches!(err, Error::InvalidGeometry(_)));
    }

    #[test]
    fn from_parts_builds_faces() {
        let verts = vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]];
        let faces = vec![vec![0u32, 1, 2], vec![0u32, 2, 3]];
        let m = PolygonMesh3D::from_parts(Coordinate::Euclidean, verts, faces).unwrap();
        assert_eq!(ones(&m.data.face_indices), vec![0, 1, 2, 0, 2, 3]);
        // Two faces -> a single internal boundary at 3.
        assert_eq!(ones(&m.data.face_offsets), vec![3]);
        assert!(ones(&m.data.interior_offsets).is_empty());
    }

    #[test]
    fn from_parts_rejects_out_of_range_index() {
        let verts = vec![[0., 0., 0.]];
        assert!(PolygonMesh3DData::from_parts(verts, vec![vec![0u32, 1, 2]]).is_err());
    }

    // An empty face contributes no ring, so it is skipped rather than producing a
    // duplicate (non-increasing) boundary.
    #[test]
    fn from_polygons_skips_empty_face() {
        let a = quad([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        let empty = Polygon3D::from_rings(
            Coordinate::Euclidean,
            Vec::<[f64; 3]>::new(),
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let b = quad([[1., 0., 0.], [2., 0., 0.], [2., 1., 0.], [1., 1., 0.]]);
        let m = PolygonMesh3DData::from_polygons([&a, &empty, &b]);
        assert_eq!(m.vertices.len(), 6);
        // Two real faces -> one internal boundary, no duplicate from the empty face.
        assert_eq!(ones(&m.face_offsets), vec![4]);
    }

    #[test]
    fn from_parts_skips_empty_face() {
        let verts = vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]];
        let faces: Vec<Vec<u32>> = vec![vec![0, 1, 2], vec![], vec![0, 2, 3]];
        let m = PolygonMesh3D::from_parts(Coordinate::Euclidean, verts, faces).unwrap();
        assert_eq!(ones(&m.data.face_indices), vec![0, 1, 2, 0, 2, 3]);
        assert_eq!(ones(&m.data.face_offsets), vec![3]);
    }

    #[test]
    fn from_parts_with_elevation_splits_z() {
        let verts = vec![[0., 0., 10.], [1., 0., 11.], [0., 1., 12.]];
        let m = PolygonMesh2D::from_parts_with_elevation(
            Coordinate::Euclidean,
            verts,
            vec![vec![0u32, 1, 2]],
        )
        .unwrap();
        assert_eq!(m.vertices, vec![[0., 0.], [1., 0.], [0., 1.]]);
        assert_eq!(m.z.as_deref(), Some(&[10., 11., 12.][..]));
    }

    #[test]
    fn from_raw_parts_validates_offsets() {
        let verts = vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.]];
        // Good: a single triangular face has no internal boundaries.
        assert!(
            PolygonMesh3DData::from_raw_parts(verts.clone(), vec![0, 1, 2], vec![], vec![]).is_ok()
        );
        // Good: two faces with one internal boundary at 3.
        assert!(PolygonMesh3DData::from_raw_parts(
            verts.clone(),
            vec![0, 1, 2, 0, 1, 2],
            vec![3],
            vec![],
        )
        .is_ok());
        // A leading 0 is no longer allowed (it would mean an empty first face).
        assert!(
            PolygonMesh3DData::from_raw_parts(verts.clone(), vec![0, 1, 2], vec![0], vec![])
                .is_err()
        );
        // Equal boundaries (an empty face) are rejected.
        assert!(PolygonMesh3DData::from_raw_parts(
            verts.clone(),
            vec![0, 1, 2, 0, 1, 2],
            vec![3, 3],
            vec![],
        )
        .is_err());
        // Index out of range.
        assert!(PolygonMesh3DData::from_raw_parts(verts, vec![0, 1, 9], vec![], vec![]).is_err());
    }

    #[test]
    fn from_raw_parts_rejects_interior_on_face_boundary() {
        // Two faces, boundary at 3; an interior offset of 3 coincides with it.
        let verts = vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.]];
        assert!(
            PolygonMesh3DData::from_raw_parts(verts, vec![0, 1, 2, 0, 1, 2], vec![3], vec![3],)
                .is_err()
        );
    }

    // ── from_polygons appearance / UV merge ──

    #[test]
    fn from_polygons_welds_single_textured_face() {
        let mut p = quad([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        p.set_appearance(theme("rgb"), textured(), Some(unit_quad_uv()))
            .unwrap();

        let m = PolygonMesh3DData::from_polygons([&p]);
        let app = m.appearance.as_ref().unwrap();
        assert_eq!(app.materials.len(), 1);
        assert_eq!(app.default_theme, theme("rgb"));
        let FaceBinding::PerFace(front) = &app.themes[0].front else {
            panic!("expected PerFace");
        };
        assert_eq!(front, &[MaterialIndex::new(0)]);
        assert!(app.themes[0].back.is_none());
        let UvSource::Explicit(uv) = &m.uv_sets[0].uv else {
            panic!("expected Explicit");
        };
        assert_eq!(uv.len(), 4);
    }

    #[test]
    fn from_polygons_concatenates_two_faces() {
        let mut a = quad([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        a.set_appearance(theme("rgb"), textured(), Some(unit_quad_uv()))
            .unwrap();
        let mut b = quad([[2., 0., 0.], [3., 0., 0.], [3., 1., 0.], [2., 1., 0.]]);
        b.set_appearance(theme("rgb"), textured(), Some(unit_quad_uv()))
            .unwrap();

        let m = PolygonMesh3DData::from_polygons([&a, &b]);
        let app = m.appearance.as_ref().unwrap();
        assert_eq!(app.materials.len(), 2);
        let FaceBinding::PerFace(front) = &app.themes[0].front else {
            panic!("expected PerFace");
        };
        assert_eq!(front, &[MaterialIndex::new(0), MaterialIndex::new(1)]);
        let UvSource::Explicit(uv) = &m.uv_sets[0].uv else {
            panic!("expected Explicit");
        };
        assert_eq!(uv.len(), 8);
    }

    #[test]
    fn from_polygons_resolves_default_theme() {
        // `a` defaults to "rgb", `b` to "infrared" — the faces diverge.
        let mut a = quad([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        a.set_appearance(theme("rgb"), bare(), None).unwrap();
        let mut b = quad([[2., 0., 0.], [3., 0., 0.], [3., 1., 0.], [2., 1., 0.]]);
        b.set_appearance(theme("infrared"), bare(), None).unwrap();

        // Auto, diverging: falls back to the first kept face's default (rule 2).
        let m = PolygonMesh3DData::from_polygons([&a, &b]);
        assert_eq!(m.appearance.as_ref().unwrap().default_theme, theme("rgb"));

        // Auto, uniform: every face agrees, so that shared default is chosen (rule 1).
        let m = PolygonMesh3DData::from_polygons([&a, &a]);
        assert_eq!(m.appearance.as_ref().unwrap().default_theme, theme("rgb"));

        // Explicit override pins the active theme regardless of the faces.
        let m =
            PolygonMesh3DData::from_polygons_with_default_theme([&a, &b], Some(theme("infrared")));
        assert_eq!(
            m.appearance.as_ref().unwrap().default_theme,
            theme("infrared")
        );
    }

    #[test]
    fn from_polygons_zero_fills_an_uncovered_face() {
        let mut a = quad([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        a.set_appearance(
            theme("rgb"),
            textured(),
            Some(explicit_uv(&[
                [0.2, 0.2],
                [0.4, 0.2],
                [0.4, 0.4],
                [0.2, 0.4],
            ])),
        )
        .unwrap();
        // `b` carries no appearance at all.
        let b = quad([[2., 0., 0.], [3., 0., 0.], [3., 1., 0.], [2., 1., 0.]]);

        let m = PolygonMesh3DData::from_polygons([&a, &b]);
        let app = m.appearance.as_ref().unwrap();
        assert_eq!(app.materials.len(), 1, "only the textured face contributes");
        let FaceBinding::PerFace(front) = &app.themes[0].front else {
            panic!("expected PerFace");
        };
        assert_eq!(front, &[MaterialIndex::new(0), None]);
        let UvSource::Explicit(uv) = &m.uv_sets[0].uv else {
            panic!("expected Explicit");
        };
        assert_eq!(uv.len(), 8);
        assert_eq!(&uv[0..4], &[[0.2, 0.2], [0.4, 0.2], [0.4, 0.4], [0.2, 0.4]]);
        assert_eq!(&uv[4..8], &[[0., 0.], [0., 0.], [0., 0.], [0., 0.]]);
    }

    #[test]
    fn from_polygons_drops_closing_uv() {
        // Closed triangle: coords [a, b, c, a] (4); the mesh keeps 3 open corners.
        let mut p = Polygon3D::from_rings(
            Coordinate::Euclidean,
            [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 0.]],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        p.set_appearance(
            theme("rgb"),
            textured(),
            Some(explicit_uv(&[[0., 0.], [1., 0.], [0., 1.], [0., 0.]])),
        )
        .unwrap();

        let m = PolygonMesh3DData::from_polygons([&p]);
        let UvSource::Explicit(uv) = &m.uv_sets[0].uv else {
            panic!("expected Explicit");
        };
        assert_eq!(uv.len(), 3);
        assert_eq!(&uv[..], &[[0., 0.], [1., 0.], [0., 1.]]);
    }

    #[test]
    fn from_polygons_bakes_world_to_texture() {
        let mut p = quad([[0., 0., 0.], [2., 0., 0.], [2., 3., 0.], [0., 3., 0.]]);
        // s = x, t = y, q = 1  ->  uv = (x, y).
        let matrix = TexMatrix([[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 0., 1.]]);
        p.set_appearance(
            theme("rgb"),
            textured(),
            Some(UvSource::WorldToTexture(matrix)),
        )
        .unwrap();

        let m = PolygonMesh3DData::from_polygons([&p]);
        let UvSource::Explicit(uv) = &m.uv_sets[0].uv else {
            panic!("expected baked Explicit");
        };
        assert_eq!(&uv[..], &[[0., 0.], [2., 0.], [2., 3.], [0., 3.]]);
    }

    #[test]
    fn from_polygons_bare_input_stays_bare() {
        let a = quad([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        let m = PolygonMesh3DData::from_polygons([&a]);
        assert!(m.appearance.is_none());
        assert!(m.uv_sets.is_empty());
    }
}
