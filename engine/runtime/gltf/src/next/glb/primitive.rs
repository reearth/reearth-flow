//! Primitive construction: [`Builder::push_primitive`] and the dedup-key
//! attribute mechanism ([`DedupAttribute`]/[`normal`]) it accepts.

use std::collections::HashMap;

use gltf::json;

use super::{Builder, Extension, MaterialDesc, PrimitiveHandle};

pub(super) struct PrimitiveBuilder {
    pub(super) positions: Vec<[f32; 3]>,
    pub(super) dedup_attrs: Vec<Box<dyn DedupAttribute>>,
    pub(super) indices: Vec<u32>,
    /// `representative[out_vertex]` is the original (pre-dedup) vertex index
    /// it was first produced from, so [`Builder::set_attribute`] can map a
    /// caller-supplied per-original-vertex array onto the deduped layout
    /// after the fact.
    pub(super) representative: Vec<u32>,
    pub(super) material: json::Index<json::Material>,
    pub(super) extra_attributes: Vec<(json::mesh::Semantic, Vec<u32>)>,
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

impl<const N: usize> AccessorValue for [f32; N] {
    fn push_accessor(values: &[Self], builder: &mut Builder) -> json::Index<json::Accessor> {
        builder.push_array_f32(values)
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
}

/// A vertex attribute that can vary between corners of the same original
/// vertex and must therefore participate in [`Builder::push_primitive`]'s
/// vertex deduplication — unlike [`Builder::set_attribute`], which only ever
/// sees data constant per original vertex. Built via [`normal`].
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
    /// Add a triangle-mesh primitive. `dedup_attrs` (see [`normal`]) are
    /// folded into vertex deduplication, splitting a vertex wherever its
    /// attributes actually disagree between corners — e.g. a flat
    /// per-polygon normal splits a vertex at a hard edge but never across an
    /// internal triangulation diagonal. `polygon_tris`/`corner_src` are the
    /// shared index arrays `dedup_attrs`' [`Granularity`] is resolved
    /// against; pass `&[]` for either if no attribute needs it.
    ///
    /// `positions`/`indices` must already be in the target coordinate
    /// convention and localized (small deltas from the translation given to
    /// [`Builder::build`], not raw ECEF).
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
        let mut representative: Vec<u32> = Vec::with_capacity(positions.len());
        let mut out_indices: Vec<u32> = Vec::with_capacity(indices.len() * 3);

        for (triangle, &[i0, i1, i2]) in indices.iter().enumerate() {
            for (corner_in_tri, &orig) in [i0, i1, i2].iter().enumerate() {
                let corner = triangle * 3 + corner_in_tri;

                value_indices.clear();
                value_indices.extend(dedup_attrs.iter().map(|attr| match attr.granularity() {
                    Granularity::PerPolygon => per_polygon_index[triangle] as usize,
                    Granularity::PerPolygonCorner => corner_src[corner] as usize,
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
                        representative.push(orig);
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

        let material_index = self.root.push(json::Material {
            pbr_metallic_roughness: json::material::PbrMetallicRoughness {
                base_color_factor: json::material::PbrBaseColorFactor(material.base_color_factor),
                metallic_factor: json::material::StrengthFactor(material.metallic_factor),
                roughness_factor: json::material::StrengthFactor(material.roughness_factor),
                ..Default::default()
            },
            ..Default::default()
        });

        self.primitives.push(PrimitiveBuilder {
            positions: out_positions,
            dedup_attrs,
            indices: out_indices,
            representative,
            material: material_index,
            extra_attributes: Vec::new(),
            extensions: Vec::new(),
        });
        PrimitiveHandle(self.primitives.len() - 1)
    }

    /// Attach an extra per-vertex scalar attribute (e.g. a feature ID) to a
    /// primitive pushed earlier. `data` is indexed by *original* vertex index
    /// (as given to [`Builder::push_primitive`]), not the deduped layout.
    pub fn set_attribute(
        &mut self,
        primitive: PrimitiveHandle,
        semantic: json::mesh::Semantic,
        data: &[u32],
    ) {
        let primitive = &mut self.primitives[primitive.0];
        let out_data: Vec<u32> = primitive
            .representative
            .iter()
            .map(|&orig| data[orig as usize])
            .collect();
        primitive.extra_attributes.push((semantic, out_data));
    }
}
