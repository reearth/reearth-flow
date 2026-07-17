//! Partition a cell's polygons by resolved material into glTF primitives.
//!
//! Material is a primitive-level selector (one material per glTF primitive), so
//! this is the coarse partition that *produces* primitives; the per-polygon
//! normals and per-corner UVs each primitive carries are vertex attributes the
//! glb builder's dedup resolves to per-vertex later. Geometry is kept in source
//! granularity and ECEF coordinates here — no per-triangle flattening — so the
//! builder deduplicates once instead of the caller expanding then re-collapsing.

use std::collections::HashMap;
use std::path::PathBuf;

use super::appearance::ResolvedMaterial;
use super::mesh::ExtractedMesh;
use reearth_flow_types::Feature;

/// Fallback for a face bound to no material: flat gray (the old writer's
/// X3DMaterial default), which keeps adjacent buildings from merging into the
/// glTF-default white.
pub(super) const DEFAULT_MATERIAL: MaterialFactors = MaterialFactors {
    base_color_factor: [0.7, 0.7, 0.7, 1.0],
    metallic_factor: 0.0,
    roughness_factor: 0.9,
};

/// UVs this far outside `[0, 1]` (the old writer's tolerance) mean a wrapping
/// texture, which can't be atlased; such a face falls back to colour-only.
const WRAP_TOLERANCE: f64 = 0.1;

/// The PBR factors that key a colour-only primitive; textures multiply these.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct MaterialFactors {
    pub(super) base_color_factor: [f32; 4],
    pub(super) metallic_factor: f32,
    pub(super) roughness_factor: f32,
}

impl MaterialFactors {
    fn of(material: Option<&ResolvedMaterial>) -> Self {
        match material {
            Some(m) => Self {
                base_color_factor: m.base_color_factor,
                metallic_factor: m.metallic_factor,
                roughness_factor: m.roughness_factor,
            },
            None => DEFAULT_MATERIAL,
        }
    }

    /// A hashable identity for grouping (f32 has no `Eq`/`Hash`).
    fn key(&self) -> [u32; 6] {
        let [r, g, b, a] = self.base_color_factor;
        [
            r.to_bits(),
            g.to_bits(),
            b.to_bits(),
            a.to_bits(),
            self.metallic_factor.to_bits(),
            self.roughness_factor.to_bits(),
        ]
    }
}

/// One primitive's geometry in source granularity and ECEF coordinates.
#[derive(Default)]
pub(super) struct Geom {
    /// Compacted vertex positions (only those this primitive's polygons use).
    pub(super) positions: Vec<[f64; 3]>,
    /// Triangle indices into `positions`.
    pub(super) indices: Vec<[u32; 3]>,
    /// One flat normal per source polygon.
    pub(super) polygon_normals: Vec<[f64; 3]>,
    /// Triangle count per source polygon, parallel to `polygon_normals`.
    pub(super) polygon_tris: Vec<u32>,
    /// Per-corner base-map UV (length `3 * indices.len()`); empty for a
    /// colour-only primitive.
    pub(super) corner_uv: Vec<[f64; 2]>,
    /// Per-vertex feature row in the tile's property table, parallel to
    /// `positions`.
    pub(super) feature_ids: Vec<u32>,
}

/// A colour-only primitive: PBR factors plus geometry.
pub(super) struct ColorPrimitive {
    pub(super) factors: MaterialFactors,
    pub(super) geom: Geom,
}

/// The single primitive aggregating every textured face in the cell; its
/// `geom.corner_uv` holds source UVs until the atlas remaps them, and
/// `polygon_texture` names each polygon's source image (parallel to
/// `geom.polygon_tris`) for the atlas pass.
pub(super) struct TexturedPrimitive {
    pub(super) geom: Geom,
    pub(super) polygon_texture: Vec<PathBuf>,
}

/// A cell's polygons partitioned into one primitive per colour-only material
/// plus the shared textured primitive.
pub(super) struct CellPrimitives {
    pub(super) color: Vec<ColorPrimitive>,
    pub(super) textured: Option<TexturedPrimitive>,
}

/// Accumulates one primitive's geometry, welding vertices by their source index
/// so the builder's dedup only has to split at genuine attribute seams.
#[derive(Default)]
struct GeomBuilder {
    geom: Geom,
    /// `(cell member, source vertex) -> local vertex` within this primitive.
    remap: HashMap<(usize, u32), u32>,
}

impl GeomBuilder {
    /// Append source polygon `p` of cell member `member` (`m`), spanning triangles
    /// `tris`, welding its vertices into this primitive. `with_uv` copies the
    /// polygon's per-corner UVs (textured primitive only).
    fn add_polygon(
        &mut self,
        member: usize,
        m: &ExtractedMesh,
        p: usize,
        tris: std::ops::Range<usize>,
        with_uv: bool,
    ) {
        for t in tris.clone() {
            let mut out = [0u32; 3];
            for (corner, &orig) in m.indices[t].iter().enumerate() {
                let local = *self.remap.entry((member, orig)).or_insert_with(|| {
                    let idx = self.geom.positions.len() as u32;
                    self.geom.positions.push(m.ecef_vertices[orig as usize]);
                    self.geom.feature_ids.push(member as u32);
                    idx
                });
                out[corner] = local;
                if with_uv {
                    self.geom.corner_uv.push(m.corner_uv[t * 3 + corner]);
                }
            }
            self.geom.indices.push(out);
        }
        self.geom.polygon_normals.push(m.polygon_normals[p]);
        self.geom.polygon_tris.push(tris.len() as u32);
    }
}

/// Partition every polygon of the cell's members into a textured bucket
/// (textured material with non-wrapping UVs) and colour-only buckets keyed by
/// PBR factors. Cross-member vertices never share, so welding is keyed by
/// `(member, source vertex)`.
pub(super) fn collect(cell_members: &[&(&Feature, ExtractedMesh)]) -> CellPrimitives {
    let mut color: HashMap<[u32; 6], (MaterialFactors, GeomBuilder)> = HashMap::new();
    let mut textured = GeomBuilder::default();
    let mut polygon_texture: Vec<PathBuf> = Vec::new();

    for (member, (_, m)) in cell_members.iter().enumerate() {
        let mut tri_off = 0usize;
        for (p, &count) in m.polygon_tris.iter().enumerate() {
            let count = count as usize;
            if count == 0 {
                continue;
            }
            let tris = tri_off..tri_off + count;
            tri_off += count;

            let material = m.triangle_material[tris.start]
                .and_then(|mi| m.materials.get(mi as usize));
            let texture = material
                .and_then(|mm| mm.base_texture.as_ref())
                .filter(|_| !polygon_wraps(&m.corner_uv[tris.start * 3..tris.end * 3]));

            match texture {
                Some(source) => {
                    textured.add_polygon(member, m, p, tris, true);
                    polygon_texture.push(source.path.clone());
                }
                None => {
                    let factors = MaterialFactors::of(material);
                    color
                        .entry(factors.key())
                        .or_insert_with(|| (factors, GeomBuilder::default()))
                        .1
                        .add_polygon(member, m, p, tris, false);
                }
            }
        }
    }

    let textured = (!textured.geom.indices.is_empty()).then_some(TexturedPrimitive {
        geom: textured.geom,
        polygon_texture,
    });
    let color = color
        .into_values()
        .map(|(factors, builder)| ColorPrimitive {
            factors,
            geom: builder.geom,
        })
        .collect();

    CellPrimitives { color, textured }
}

/// Whether any of a polygon's corner UVs falls outside `[0, 1]` (with tolerance).
fn polygon_wraps(uvs: &[[f64; 2]]) -> bool {
    let unit = -WRAP_TOLERANCE..=1.0 + WRAP_TOLERANCE;
    uvs.iter()
        .any(|&[u, v]| !unit.contains(&u) || !unit.contains(&v))
}
