//! Shared geometry: writing one `gml:Polygon` once and pointing at it.
//!
//! A CityGML LOD2 building describes its faces twice. Each face is a
//! `gml:Polygon` under the boundary surface that names it a wall, a roof or a
//! ground, and the same face is a member of the shell of `lod2Solid`. PLATEAU
//! writes the polygon once, under the boundary surface, and gives the solid an
//! `xlink:href` to its `gml:id`; re-serializing the coordinates in both places
//! doubles the file and loses the fact that the two are one surface.
//!
//! The writer finds the repeats by comparing the surfaces it is about to
//! write, which is the whole of the agreed design: the geometry model carries
//! no ownership and nothing new is minted per face. Two surfaces are the same
//! when their coordinates are equal *and* their orientation is the same, so a
//! ring and its reverse stay two surfaces — in CityGML they face opposite ways
//! and mean different things.
//!
//! Sameness also takes in what the surface carries, not only where it is:
//! its appearance and its source `gml:id`. Writing one polygon leaves one
//! `gml:id` for an `app:surfaceDataMember` to target, so two copies painted
//! differently cannot both survive the merge — and a tunnel really does carry
//! one face at LOD3 and LOD4 under different textures. Comparing the paint as
//! well as the coordinates makes sharing lossless by construction instead of
//! making it something to warn about: what is merged is indistinguishable, and
//! what differs stays two surfaces.
//!
//! The plan is built once per city object tree, over the settled tree, so it
//! names only surfaces that actually reach the document. Everything it decides
//! is then read back during the write in [`SharedSurfaces::site`].

use std::collections::HashMap;

use super::model::{GeometryEntry, GmlElement, GmlSurface};

/// One polygon's geometry, as the writer compares two of them.
///
/// Each ordinate is kept by its bit pattern, so the comparison is exact: a
/// coordinate that survived the conversion unchanged matches, and one that did
/// not is a different surface rather than a near miss. Rings without UVs are
/// rotated to a canonical first vertex, which absorbs the one difference that
/// carries no meaning — where a closed ring starts — while leaving direction
/// alone. A ring with UVs keeps its start, because there the start does mean
/// something: it is what pairs each vertex with its texture coordinate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct SurfaceKey {
    /// Exterior first, then each interior in its written order.
    rings: Vec<Vec<[u64; 3]>>,
    /// The palettes an index means are per source, so the source is part of
    /// the identity: index 0 is a different material under another feature.
    source: usize,
    material_idx: Option<u32>,
    texture_idx: Option<u32>,
    /// Where the texture sits on the face. Empty when nothing is textured.
    uv: Vec<Vec<[u64; 2]>>,
    /// Two coincident polygons the source named apart are two surfaces, and
    /// merging them would have to throw one name away.
    id: Option<String>,
}

impl SurfaceKey {
    /// `None` for a surface that cannot be compared: a ring with a non-finite
    /// ordinate, or one too short to be a polygon. Those are written in full
    /// every time, which is what the writer did before sharing existed.
    pub(super) fn of(surface: &GmlSurface, source: usize) -> Option<Self> {
        // UVs are listed one per vertex, so a ring that carries them keeps its
        // starting vertex: rotating the ring alone would make two different
        // texturings compare equal. A material, or a texture with no UVs,
        // covers the whole face and is safe to rotate under.
        let rotate =
            surface.uv_exterior.is_empty() && surface.uv_interiors.iter().all(Vec::is_empty);
        let mut rings = Vec::with_capacity(1 + surface.interiors.len());
        rings.push(canonical_ring(&surface.exterior, rotate)?);
        for interior in &surface.interiors {
            rings.push(canonical_ring(interior, rotate)?);
        }
        let mut uv = Vec::with_capacity(1 + surface.uv_interiors.len());
        uv.push(canonical_uv(&surface.uv_exterior)?);
        for ring in &surface.uv_interiors {
            uv.push(canonical_uv(ring)?);
        }
        Some(Self {
            rings,
            source,
            material_idx: surface.material_idx,
            texture_idx: surface.texture_idx,
            uv,
            id: surface.id.clone(),
        })
    }
}

/// How good a place one occurrence of a surface is to carry the full polygon,
/// compared field by field in this order and maximised.
///
/// The first occurrence in document order is not the right one to write in
/// full. `lod2Solid` is declared before `boundedBy` in `_AbstractBuilding`, so
/// taking the first would move every face out of the boundary surface that
/// names it and into the solid — the opposite of the shape PLATEAU uses and of
/// what a reader expects to find. Depth puts the polygon back under the
/// boundary surface, and `outside_solid` settles the case where one object
/// carries both a solid and a multi-surface at the same LOD.
///
/// Appearance is not a tiebreaker here: every occurrence in one group carries
/// the same paint, because the paint is part of what makes them one surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Rank {
    depth: u16,
    outside_solid: bool,
}

/// Where one surface is being written, which is all [`Rank`] is built from.
#[derive(Debug, Clone, Copy)]
pub(super) struct Site {
    /// Nesting depth of the object the surface belongs to, 0 for the
    /// top-level city object.
    pub(super) depth: u16,
    /// Whether the surface is a face of a `gml:Solid`.
    pub(super) in_solid: bool,
    /// The feature the owning object reports against, so a dropped appearance
    /// is reported against the right one.
    pub(super) source: usize,
}

impl Site {
    fn rank(&self) -> Rank {
        Rank {
            depth: self.depth,
            outside_solid: !self.in_solid,
        }
    }
}

/// What the plan decided about one surface written more than once.
#[derive(Debug)]
struct Entry {
    /// The `gml:id` the full polygon is written with and every repeat points
    /// at. Taken from the source when a surface carries one.
    id: String,
    /// The rank the definition occurrence has.
    best: Rank,
    /// Set once the full polygon has been written, so of several occurrences
    /// tied at `best` only the first writes one.
    written: bool,
}

/// What to do with the surface about to be written.
pub(super) enum Decision {
    /// Not shared: write the polygon as the writer always has.
    Alone,
    /// The first and best-placed occurrence: write the polygon in full, under
    /// this `gml:id`, for the repeats to point at.
    Define(String),
    /// A repeat: write `<gml:surfaceMember xlink:href="#id"/>`. Nothing is
    /// lost by it: a repeat is indistinguishable from the definition, paint
    /// and source `gml:id` included.
    Reference(String),
}

/// The surfaces one city object tree writes more than once, and the `gml:id`
/// each is written under.
#[derive(Debug, Default)]
pub(super) struct SharedSurfaces {
    entries: HashMap<SurfaceKey, Entry>,
}

impl SharedSurfaces {
    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Decide what to write for `surface` at `site`, and record that the
    /// definition has been written.
    pub(super) fn site(&mut self, surface: &GmlSurface, site: &Site) -> Decision {
        let Some(key) = SurfaceKey::of(surface, site.source) else {
            return Decision::Alone;
        };
        let Some(entry) = self.entries.get_mut(&key) else {
            return Decision::Alone;
        };
        if !entry.written && site.rank() == entry.best {
            entry.written = true;
            return Decision::Define(entry.id.clone());
        }
        Decision::Reference(entry.id.clone())
    }
}

/// One surface written more than once in a tree, before the writer has settled
/// its `gml:id`.
pub(super) struct Draft {
    pub(super) key: SurfaceKey,
    /// The `gml:id` the source gave the surface, if any; the writer mints one
    /// when there is none. Every occurrence in a group carries the same one,
    /// since the id is part of what makes them one surface.
    pub(super) candidate_id: Option<String>,
    best: Rank,
}

/// A node of the settled tree the plan walks: whatever the writer is about to
/// write, in the order it will write it.
pub(super) trait SurfaceTree {
    fn source(&self) -> usize;
    fn entries(&self) -> &[GeometryEntry];
    fn children(&self) -> Vec<&Self>;
}

/// Find every surface a tree writes more than once, in the order the tree
/// first shows each.
///
/// The order matters only so that the ids the writer mints run the same way
/// twice for the same input.
pub(super) fn plan<T: SurfaceTree>(root: &T) -> Vec<Draft> {
    let mut order: Vec<SurfaceKey> = Vec::new();
    let mut seen: HashMap<SurfaceKey, (usize, Draft)> = HashMap::new();
    walk(root, 0, &mut |surface, site| {
        let Some(key) = SurfaceKey::of(surface, site.source) else {
            return;
        };
        let rank = site.rank();
        match seen.get_mut(&key) {
            Some((count, draft)) => {
                *count += 1;
                // Strictly greater, so of several occurrences ranked alike the
                // first the walk meets is the definition.
                if rank > draft.best {
                    draft.best = rank;
                }
            }
            None => {
                order.push(key.clone());
                seen.insert(
                    key.clone(),
                    (
                        1,
                        Draft {
                            key,
                            candidate_id: surface.id.clone(),
                            best: rank,
                        },
                    ),
                );
            }
        }
    });

    order
        .into_iter()
        .filter_map(|key| match seen.remove(&key) {
            Some((count, draft)) if count > 1 => Some(draft),
            _ => None,
        })
        .collect()
}

impl SharedSurfaces {
    /// Bind each draft to the `gml:id` the writer settled for it.
    pub(super) fn from_drafts(drafts: impl IntoIterator<Item = (Draft, String)>) -> Self {
        Self {
            entries: drafts
                .into_iter()
                .map(|(draft, id)| {
                    (
                        draft.key,
                        Entry {
                            id,
                            best: draft.best,
                            written: false,
                        },
                    )
                })
                .collect(),
        }
    }
}

/// Visit every surface of `node` and the objects nested in it, in writing
/// order.
fn walk<T: SurfaceTree>(node: &T, depth: u16, visit: &mut impl FnMut(&GmlSurface, Site)) {
    let source = node.source();
    for entry in node.entries() {
        let mut face = |surface: &GmlSurface, in_solid: bool| {
            visit(
                surface,
                Site {
                    depth,
                    in_solid,
                    source,
                },
            )
        };
        match &entry.element {
            GmlElement::Solid(solid) => {
                for surface in solid
                    .exterior
                    .iter()
                    .chain(solid.interiors.iter().flatten())
                {
                    face(surface, true);
                }
            }
            GmlElement::MultiSolid { solids, .. } => {
                for solid in solids {
                    for surface in solid
                        .exterior
                        .iter()
                        .chain(solid.interiors.iter().flatten())
                    {
                        face(surface, true);
                    }
                }
            }
            GmlElement::MultiSurface { surfaces, .. } => {
                for surface in surfaces {
                    face(surface, false);
                }
            }
            // A curve has no surface to share.
            GmlElement::MultiCurve { .. } => {}
        }
    }
    for child in node.children() {
        walk(child, depth + 1, visit);
    }
}

/// One ring's texture coordinates by their bit patterns, in the order they
/// were recorded. `None` on a non-finite ordinate, which makes the surface
/// incomparable exactly as a non-finite position does.
///
/// These are never rotated, and neither is a ring that has them: see
/// [`SurfaceKey::of`]. Two copies of one textured face read from different
/// starting vertices are therefore two surfaces. That gives up some sharing,
/// never any data, which is the right side to err on.
fn canonical_uv(uv: &[[f64; 2]]) -> Option<Vec<[u64; 2]>> {
    let mut bits = Vec::with_capacity(uv.len());
    for point in uv {
        let mut coord = [0u64; 2];
        for (slot, ordinate) in coord.iter_mut().zip(point.iter()) {
            if !ordinate.is_finite() {
                return None;
            }
            *slot = (ordinate + 0.0).to_bits();
        }
        bits.push(coord);
    }
    Some(bits)
}

/// One ring as it is compared: closed back to open, every ordinate by its bit
/// pattern, and rotated to a canonical first vertex when `rotate` is set.
///
/// `None` when a ring cannot be compared. A non-finite ordinate has no usable
/// bit pattern — `NaN != NaN`, and several bit patterns are `NaN` — and a ring
/// of fewer than three vertices is not a polygon boundary.
fn canonical_ring(ring: &[[f64; 3]], rotate: bool) -> Option<Vec<[u64; 3]>> {
    let mut bits: Vec<[u64; 3]> = Vec::with_capacity(ring.len());
    for point in ring {
        let mut coord = [0u64; 3];
        for (slot, ordinate) in coord.iter_mut().zip(point.iter()) {
            if !ordinate.is_finite() {
                return None;
            }
            // `-0.0` and `0.0` are the same position with different bits.
            *slot = (ordinate + 0.0).to_bits();
        }
        bits.push(coord);
    }
    // A ring arrives closed; the repeated last vertex says nothing about which
    // ring this is, and leaving it in would make a rotation look different.
    if bits.len() > 1 && bits[0] == bits[bits.len() - 1] {
        bits.pop();
    }
    if bits.len() < 3 {
        return None;
    }
    Some(if rotate {
        rotate_to_canonical(bits)
    } else {
        bits
    })
}

/// Rotate a ring so it starts at the vertex that makes the sequence smallest,
/// which gives every rotation of one ring the same representation and leaves
/// the direction it runs in untouched.
///
/// A ring can repeat its smallest vertex, so the candidates are compared as
/// whole sequences rather than by their first vertex alone.
fn rotate_to_canonical(ring: Vec<[u64; 3]>) -> Vec<[u64; 3]> {
    let smallest = match ring.iter().min() {
        Some(smallest) => *smallest,
        None => return ring,
    };
    let rotation = |at: usize| ring[at..].iter().chain(ring[..at].iter()).copied();
    let best = (0..ring.len())
        .filter(|at| ring[*at] == smallest)
        .min_by(|left, right| rotation(*left).cmp(rotation(*right)))
        .unwrap_or(0);
    rotation(best).collect()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    fn surface(exterior: Vec<[f64; 3]>) -> GmlSurface {
        GmlSurface {
            id: None,
            exterior,
            interiors: Vec::new(),
            material_idx: None,
            texture_idx: None,
            uv_exterior: Vec::new(),
            uv_interiors: Vec::new(),
        }
    }

    fn triangle() -> Vec<[f64; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ]
    }

    #[test]
    fn the_same_ring_from_a_different_starting_vertex_is_one_surface() {
        let rotated = vec![
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ];
        assert_eq!(
            SurfaceKey::of(&surface(triangle()), 0),
            SurfaceKey::of(&surface(rotated), 0),
        );
    }

    #[test]
    fn a_reversed_ring_is_a_different_surface() {
        let mut reversed = triangle();
        reversed.reverse();
        assert_ne!(
            SurfaceKey::of(&surface(triangle()), 0),
            SurfaceKey::of(&surface(reversed), 0),
        );
    }

    #[test]
    fn an_open_ring_matches_the_closed_one() {
        let open = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        assert_eq!(
            SurfaceKey::of(&surface(triangle()), 0),
            SurfaceKey::of(&surface(open), 0),
        );
    }

    #[test]
    fn negative_zero_is_the_same_position_as_zero() {
        let mut signed = triangle();
        signed[0] = [-0.0, -0.0, -0.0];
        signed[3] = [-0.0, -0.0, -0.0];
        assert_eq!(
            SurfaceKey::of(&surface(triangle()), 0),
            SurfaceKey::of(&surface(signed), 0),
        );
    }

    #[test]
    fn a_ring_with_a_non_finite_ordinate_is_never_shared() {
        let mut broken = triangle();
        broken[1] = [f64::NAN, 0.0, 0.0];
        assert!(SurfaceKey::of(&surface(broken), 0).is_none());
    }

    #[test]
    fn a_ring_of_two_vertices_is_never_shared() {
        let sliver = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        assert!(SurfaceKey::of(&surface(sliver), 0).is_none());
    }

    #[test]
    fn holes_are_part_of_what_makes_a_surface_the_same() {
        let hole = vec![
            [0.1, 0.1, 0.0],
            [0.2, 0.1, 0.0],
            [0.2, 0.2, 0.0],
            [0.1, 0.1, 0.0],
        ];
        let mut with_hole = surface(triangle());
        with_hole.interiors = vec![hole];
        assert_ne!(
            SurfaceKey::of(&surface(triangle()), 0),
            SurfaceKey::of(&with_hole, 0),
        );
    }

    /// The smallest vertex appearing twice must still give one rotation, or a
    /// ring would compare unequal to a rotation of itself.
    #[test]
    fn a_repeated_smallest_vertex_still_rotates_the_same_way() {
        let bow_tie = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let from_the_other_repeat = vec![
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ];
        assert_eq!(
            SurfaceKey::of(&surface(bow_tie), 0),
            SurfaceKey::of(&surface(from_the_other_repeat), 0),
        );
    }

    #[test]
    fn a_differently_painted_copy_is_a_different_surface() {
        let mut painted = surface(triangle());
        painted.material_idx = Some(1);
        assert_ne!(
            SurfaceKey::of(&surface(triangle()), 0),
            SurfaceKey::of(&painted, 0),
        );
    }

    /// Palettes are per source, so the same index under another feature means
    /// another material.
    #[test]
    fn the_same_index_under_another_source_is_a_different_surface() {
        let mut painted = surface(triangle());
        painted.material_idx = Some(0);
        assert_ne!(SurfaceKey::of(&painted, 0), SurfaceKey::of(&painted, 1),);
    }

    #[test]
    fn the_same_texture_laid_on_differently_is_a_different_surface() {
        let mut one = surface(triangle());
        one.texture_idx = Some(0);
        one.uv_exterior = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let mut other = one.clone();
        other.uv_exterior = vec![[0.0, 0.0], [0.5, 0.0], [0.5, 0.5]];
        assert_ne!(SurfaceKey::of(&one, 0), SurfaceKey::of(&other, 0));
    }

    /// Two coincident polygons the source named apart stay two surfaces,
    /// because merging them would have to throw one `gml:id` away.
    #[test]
    fn copies_the_source_named_apart_are_different_surfaces() {
        let mut one = surface(triangle());
        one.id = Some("poly-a".to_owned());
        let mut other = surface(triangle());
        other.id = Some("poly-b".to_owned());
        assert_ne!(SurfaceKey::of(&one, 0), SurfaceKey::of(&other, 0));
    }

    /// The ring `[a, b, c]` and its rotation `[b, c, a]` under the same UV
    /// list pair different vertices with each texture coordinate: `a` takes
    /// the first UV in one and `b` in the other. Rotating the rings alone made
    /// those two texturings compare equal, so one was silently dropped.
    #[test]
    fn a_rotated_ring_under_the_same_uv_list_is_a_different_surface() {
        let uv = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]];
        let mut one = surface(triangle());
        one.texture_idx = Some(0);
        one.uv_exterior = uv.clone();
        let mut rotated = surface(vec![
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ]);
        rotated.texture_idx = Some(0);
        rotated.uv_exterior = uv;
        assert_ne!(SurfaceKey::of(&one, 0), SurfaceKey::of(&rotated, 0));
    }

    /// Rotating the UVs along with the ring gives the same texturing, but it
    /// is still left as two surfaces. Matching it would mean rotating the UVs
    /// in step, which is more machinery than the sharing it buys; the cost of
    /// not matching is a polygon written twice, never a texture lost.
    #[test]
    fn a_textured_ring_is_not_matched_across_rotations() {
        let mut one = surface(triangle());
        one.texture_idx = Some(0);
        one.uv_exterior = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]];
        let mut rotated = surface(vec![
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ]);
        rotated.texture_idx = Some(0);
        rotated.uv_exterior = vec![[1.0, 0.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0]];
        assert_ne!(SurfaceKey::of(&one, 0), SurfaceKey::of(&rotated, 0));
    }

    /// A material covers the whole face, so nothing pairs with a vertex and a
    /// painted ring still matches its rotations.
    #[test]
    fn a_material_painted_ring_still_matches_its_rotations() {
        let mut one = surface(triangle());
        one.material_idx = Some(0);
        let mut rotated = surface(vec![
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ]);
        rotated.material_idx = Some(0);
        assert_eq!(SurfaceKey::of(&one, 0), SurfaceKey::of(&rotated, 0));
    }
}
