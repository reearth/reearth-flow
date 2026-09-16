//! Primitive construction: [`Builder::push_primitive`] and the dedup-key
//! attribute mechanism ([`DedupAttribute`]/[`normal`]) it accepts.

use std::collections::HashMap;

use gltf::json;

use super::{Builder, Extension, MaterialDesc, PrimitiveHandle};

/// A per-corner UV (`TEXCOORD_0`), folded into [`Builder::push_primitive`]'s
/// vertex dedup so corners sharing a position/normal but differing in UV are
/// kept distinct. `values` is one entry per polygon-vertex, resolved against
/// `push_primitive`'s `corner_src`.
pub fn texcoord(values: Vec<[f32; 2]>) -> Box<dyn DedupAttribute> {
    Box::new(DedupValue {
        semantic: json::mesh::Semantic::TexCoords(0),
        granularity: Granularity::PerPolygonCorner,
        values,
        out_values: Vec::new(),
    })
}

pub(super) struct PrimitiveBuilder {
    pub(super) positions: Vec<[f32; 3]>,
    pub(super) dedup_attrs: Vec<Box<dyn DedupAttribute>>,
    pub(super) indices: Vec<u32>,
    pub(super) material: json::Index<json::Material>,
    pub(super) extensions: Vec<Extension>,
}

/// Canonical hashable bits for a vertex-attribute value, used to fold it into
/// [`Builder::push_primitive`]'s dedup key.
trait KeyBits: Copy {
    fn push_bits(self, into: &mut Vec<u32>);
}

impl<const N: usize> KeyBits for [f32; N] {
    fn push_bits(self, into: &mut Vec<u32>) {
        into.extend(self.map(f32::to_bits));
    }
}

/// A vertex-attribute value type: knows how to serialize itself into an
/// accessor, on top of [`KeyBits`]'s hashing.
trait AccessorValue: KeyBits {
    fn push_accessor(values: &[Self], builder: &mut Builder) -> json::Index<json::Accessor>;
}

impl KeyBits for u32 {
    fn push_bits(self, into: &mut Vec<u32>) {
        into.push(self);
    }
}

impl<const N: usize> AccessorValue for [f32; N] {
    fn push_accessor(values: &[Self], builder: &mut Builder) -> json::Index<json::Accessor> {
        builder.push_array_f32(values)
    }
}

impl AccessorValue for u32 {
    fn push_accessor(values: &[Self], builder: &mut Builder) -> json::Index<json::Accessor> {
        builder.push_scalar_u32(values, json::buffer::Target::ArrayBuffer)
    }
}

/// Which shared per-triangle/per-corner index a dedup-key attribute's value
/// is looked up through; resolved centrally by [`Builder::push_primitive`].
#[derive(Clone, Copy)]
pub enum Granularity {
    /// One value per source polygon, looked up via `push_primitive`'s
    /// `polygon_tris`.
    PerPolygon,
    /// One value per polygon-vertex, looked up via `push_primitive`'s
    /// `corner_src`.
    PerPolygonCorner,
    /// One value per original vertex, looked up by the vertex index itself.
    PerVertex,
}

/// A vertex attribute participating in [`Builder::push_primitive`]'s vertex
/// deduplication. Built via [`normal`], [`texcoord`] or [`scalar_u32`].
pub trait DedupAttribute {
    fn granularity(&self) -> Granularity;
    fn push_key_bits(&self, value_index: usize, into: &mut Vec<u32>);
    fn commit(&mut self, value_index: usize);
    fn into_accessor(
        self: Box<Self>,
        builder: &mut Builder,
    ) -> (json::mesh::Semantic, json::Index<json::Accessor>);
}

struct DedupValue<T: AccessorValue> {
    semantic: json::mesh::Semantic,
    granularity: Granularity,
    /// One entry per polygon, or per polygon-vertex (per [`Granularity`]) —
    /// never per triangle corner.
    values: Vec<T>,
    out_values: Vec<T>,
}

impl<T: AccessorValue> DedupAttribute for DedupValue<T> {
    fn granularity(&self) -> Granularity {
        self.granularity
    }
    fn push_key_bits(&self, value_index: usize, into: &mut Vec<u32>) {
        self.values[value_index].push_bits(into);
    }
    fn commit(&mut self, value_index: usize) {
        self.out_values.push(self.values[value_index]);
    }
    fn into_accessor(
        self: Box<Self>,
        builder: &mut Builder,
    ) -> (json::mesh::Semantic, json::Index<json::Accessor>) {
        (self.semantic, T::push_accessor(&self.out_values, builder))
    }
}

/// A flat per-polygon (or, for a seam, per-polygon-corner) normal, folded
/// into [`Builder::push_primitive`]'s vertex dedup: `values` is compact
/// (never per triangle corner), resolved against `granularity`'s matching
/// shared index.
pub fn normal(granularity: Granularity, values: Vec<[f32; 3]>) -> Box<dyn DedupAttribute> {
    Box::new(DedupValue {
        semantic: json::mesh::Semantic::Normals,
        granularity,
        values,
        out_values: Vec::new(),
    })
}

/// A `u32` scalar under the custom attribute `name` (emitted as `_NAME`; the
/// glTF-spec-mandated leading underscore is added on serialization), folded
/// into [`Builder::push_primitive`]'s vertex dedup like any other attribute:
/// vertices disagreeing on it are never welded.
pub fn scalar_u32(name: &str, granularity: Granularity, values: Vec<u32>) -> Box<dyn DedupAttribute> {
    Box::new(DedupValue {
        semantic: json::mesh::Semantic::Extras(name.to_string()),
        granularity,
        values,
        out_values: Vec::new(),
    })
}

/// One entry per *triangle* (not per corner, not per polygon):
/// `polygon_tris[p]` consecutive triangles all get polygon index `p`.
fn expand_polygon_index(polygon_tris: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(polygon_tris.iter().sum::<u32>() as usize);
    for (polygon, &count) in polygon_tris.iter().enumerate() {
        out.extend(std::iter::repeat_n(polygon as u32, count as usize));
    }
    out
}

impl Builder {
    /// Add a triangle-mesh primitive.
    ///
    /// `positions`/`indices`: already localized, in target convention.
    /// `dedup_attrs` (see [`normal`]): folded into the dedup key, splitting a vertex where corners disagree.
    /// `polygon_tris`/`corner_src`: index arrays `dedup_attrs`' [`Granularity`] resolves against; `&[]` if unused.
    pub fn push_primitive(
        &mut self,
        positions: Vec<[f32; 3]>,
        indices: Vec<[u32; 3]>,
        material: MaterialDesc,
        polygon_tris: &[u32],
        corner_src: &[u32],
        mut dedup_attrs: Vec<Box<dyn DedupAttribute>>,
    ) -> PrimitiveHandle {
        let per_polygon_index = expand_polygon_index(polygon_tris);

        let mut dedup: HashMap<Vec<u32>, u32> = HashMap::new();
        let mut scratch: Vec<u32> = Vec::new();
        let mut value_indices: Vec<usize> = Vec::with_capacity(dedup_attrs.len());
        let mut out_positions: Vec<[f32; 3]> = Vec::with_capacity(positions.len());
        let mut out_indices: Vec<u32> = Vec::with_capacity(indices.len() * 3);

        for (triangle, &[i0, i1, i2]) in indices.iter().enumerate() {
            for (corner_in_tri, &orig) in [i0, i1, i2].iter().enumerate() {
                let corner = triangle * 3 + corner_in_tri;

                value_indices.clear();
                value_indices.extend(dedup_attrs.iter().map(|attr| match attr.granularity() {
                    Granularity::PerPolygon => per_polygon_index[triangle] as usize,
                    Granularity::PerPolygonCorner => corner_src[corner] as usize,
                    Granularity::PerVertex => orig as usize,
                }));

                let p = positions[orig as usize];
                scratch.clear();
                scratch.extend(p.map(f32::to_bits));
                for (attr, &vi) in dedup_attrs.iter().zip(&value_indices) {
                    attr.push_key_bits(vi, &mut scratch);
                }

                let idx = match dedup.get(scratch.as_slice()) {
                    Some(&idx) => idx,
                    None => {
                        let idx = out_positions.len() as u32;
                        out_positions.push(p);
                        for (attr, &vi) in dedup_attrs.iter_mut().zip(&value_indices) {
                            attr.commit(vi);
                        }
                        dedup.insert(scratch.clone(), idx);
                        idx
                    }
                };
                out_indices.push(idx);
            }
        }

        let base_color_texture = material.base_color_texture.map(|tex| json::texture::Info {
            index: tex.index(),
            tex_coord: 0,
            extensions: Default::default(),
            extras: Default::default(),
        });
        let material_index = self.root.push(json::Material {
            pbr_metallic_roughness: json::material::PbrMetallicRoughness {
                base_color_factor: json::material::PbrBaseColorFactor(material.base_color_factor),
                metallic_factor: json::material::StrengthFactor(material.metallic_factor),
                roughness_factor: json::material::StrengthFactor(material.roughness_factor),
                base_color_texture,
                ..Default::default()
            },
            ..Default::default()
        });

        self.primitives.push(PrimitiveBuilder {
            positions: out_positions,
            dedup_attrs,
            indices: out_indices,
            material: material_index,
            extensions: Vec::new(),
        });
        PrimitiveHandle(self.primitives.len() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn material() -> MaterialDesc {
        MaterialDesc {
            base_color_factor: [1.0, 1.0, 1.0, 1.0],
            metallic_factor: 0.0,
            roughness_factor: 1.0,
            base_color_texture: None,
        }
    }

    /// Two triangles sharing an edge (original vertices 1 and 2), each in its
    /// own polygon with its own per-polygon normal.
    fn two_triangles_sharing_an_edge() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let indices = vec![[0, 1, 2], [2, 1, 3]];
        (positions, indices)
    }

    #[test]
    fn test_vertex_dedup_with_sharp_edge() {
        let (positions, indices) = two_triangles_sharing_an_edge();
        let dedup = vec![normal(
            Granularity::PerPolygon,
            vec![[0.0, 0.0, 1.0], [0.0, 0.0, -1.0]],
        )];

        let mut builder = Builder::new();
        let handle = builder.push_primitive(positions, indices, material(), &[1, 1], &[], dedup);

        // Vertices 1 and 2 are shared by position, but each triangle's flat
        // normal disagrees on them, so none of the 6 corners may collapse.
        assert_eq!(builder.primitives[handle.0].positions.len(), 6);
    }

    #[test]
    fn test_vertex_dedup_with_smooth_edge() {
        let (positions, indices) = two_triangles_sharing_an_edge();
        let dedup = vec![normal(
            Granularity::PerPolygon,
            vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
        )];

        let mut builder = Builder::new();
        let handle = builder.push_primitive(positions, indices, material(), &[1, 1], &[], dedup);

        // Same shared vertices, but both polygons agree on the normal now, so
        // the shared corners collapse back to the 4 original positions.
        assert_eq!(builder.primitives[handle.0].positions.len(), 4);
    }

    /// Two features meeting along an edge: each carries its own copy of the
    /// two shared corners, so no original vertex belongs to both features.
    /// Positions and per-polygon normals agree on those corners.
    fn two_features_meeting_at_an_edge() -> (Vec<[f32; 3]>, Vec<[u32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let indices = vec![[0, 1, 2], [3, 4, 5]];
        let feature_ids = vec![0, 0, 0, 1, 1, 1];
        (positions, indices, feature_ids)
    }

    // A triangle belongs to exactly one feature, so every vertex a triangle
    // references must carry that feature's ID. Dedup may not collapse two
    // vertices of different features onto one, since the survivor's ID would
    // then be read for the other feature's triangle too.
    #[test]
    fn dedup_keeps_vertices_of_different_features_apart() {
        let (positions, indices, feature_ids) = two_features_meeting_at_an_edge();
        let dedup = vec![
            normal(
                Granularity::PerPolygon,
                vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
            ),
            scalar_u32("FEATURE_ID_0", Granularity::PerVertex, feature_ids.clone()),
        ];

        let mut builder = Builder::new();
        let handle = builder.push_primitive(positions, indices, material(), &[1, 1], &[], dedup);

        // Position and normal agree on the two shared corners, so without the
        // feature ID in the key they would collapse to 4 and feature 1's
        // triangle would read feature 0's ID.
        assert_eq!(builder.primitives[handle.0].positions.len(), 6);
    }
}
