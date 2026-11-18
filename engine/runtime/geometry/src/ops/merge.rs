//! Merging polygons and meshes into one mesh.
//!
//! Faces are copied as they are, with no re-triangulation. Vertices with the
//! same coordinates (exact `f64` bits) are welded into one shared pool.

use std::collections::{HashMap, HashSet};

use crate::appearance::{
    Appearance, ChannelId, FaceBinding, Material, MaterialIndex, Side, ThemeBinding, ThemeId,
    UvSet, UvSource,
};
use crate::coordinate::CoordinateFrame;
use crate::error::Error;
use crate::index::IndexBuffer;
use crate::polygon_mesh::{PolygonMesh2D, PolygonMesh3D, bake};
use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// Whether [`merge_meshes`] accepts `geometry`: a polygon, a polygon mesh, a
/// triangular mesh, or a collection of them, in either dimension.
pub fn is_mergeable(geometry: &Geometry) -> bool {
    match geometry {
        Geometry::None => false,
        Geometry::Euclidean2D(g) => mergeable_2d(g),
        Geometry::Euclidean3D(g) => mergeable_3d(g),
        Geometry::GeometryCollection(c) => {
            !c.members().is_empty() && c.members().iter().all(is_mergeable)
        }
    }
}

fn mergeable_2d(geometry: &Euclidean2DGeometry) -> bool {
    match geometry {
        Euclidean2DGeometry::Polygon(_)
        | Euclidean2DGeometry::PolygonMesh(_)
        | Euclidean2DGeometry::TriangularMesh(_) => true,
        Euclidean2DGeometry::Collection(c) => {
            !c.members().is_empty() && c.members().iter().all(mergeable_2d)
        }
        _ => false,
    }
}

fn mergeable_3d(geometry: &Euclidean3DGeometry) -> bool {
    match geometry {
        Euclidean3DGeometry::Polygon(_)
        | Euclidean3DGeometry::PolygonMesh(_)
        | Euclidean3DGeometry::TriangularMesh(_) => true,
        Euclidean3DGeometry::Collection(c) => {
            !c.members().is_empty() && c.members().iter().all(mergeable_3d)
        }
        _ => false,
    }
}

/// Merge every face of `geometries` into one mesh.
///
/// The result is a triangular mesh when every input is a triangular mesh, and a
/// polygon mesh otherwise. Faces keep their input order, and appearance is
/// carried over: palettes concatenate, themes union, and each theme's UV is
/// rebuilt per corner. Errors when an input is not
/// [mergeable](is_mergeable), or when the inputs differ in dimension, frame, or
/// 2D elevation. No faces yield [`Geometry::None`].
pub fn merge_meshes<'a>(
    geometries: impl IntoIterator<Item = &'a Geometry>,
) -> Result<Geometry, Error> {
    let mut leaves_2d = Vec::new();
    let mut leaves_3d = Vec::new();
    for geometry in geometries {
        collect(geometry, &mut leaves_2d, &mut leaves_3d)?;
    }
    match (leaves_2d.is_empty(), leaves_3d.is_empty()) {
        (true, true) => Ok(Geometry::None),
        (false, true) => merge_2d(&leaves_2d),
        (true, false) => merge_3d(&leaves_3d),
        (false, false) => Err(Error::invalid_geometry(
            "cannot merge 2D and 3D geometries into one mesh",
        )),
    }
}

fn collect<'a>(
    geometry: &'a Geometry,
    leaves_2d: &mut Vec<&'a Euclidean2DGeometry>,
    leaves_3d: &mut Vec<&'a Euclidean3DGeometry>,
) -> Result<(), Error> {
    match geometry {
        Geometry::None => Ok(()),
        Geometry::Euclidean2D(g) => collect_2d(g, leaves_2d),
        Geometry::Euclidean3D(g) => collect_3d(g, leaves_3d),
        Geometry::GeometryCollection(c) => c
            .members()
            .iter()
            .try_for_each(|m| collect(m, leaves_2d, leaves_3d)),
    }
}

fn collect_2d<'a>(
    geometry: &'a Euclidean2DGeometry,
    leaves: &mut Vec<&'a Euclidean2DGeometry>,
) -> Result<(), Error> {
    match geometry {
        Euclidean2DGeometry::Collection(c) => {
            c.members().iter().try_for_each(|m| collect_2d(m, leaves))
        }
        g if mergeable_2d(g) => {
            leaves.push(g);
            Ok(())
        }
        _ => Err(unsupported()),
    }
}

fn collect_3d<'a>(
    geometry: &'a Euclidean3DGeometry,
    leaves: &mut Vec<&'a Euclidean3DGeometry>,
) -> Result<(), Error> {
    match geometry {
        Euclidean3DGeometry::Collection(c) => {
            c.members().iter().try_for_each(|m| collect_3d(m, leaves))
        }
        g if mergeable_3d(g) => {
            leaves.push(g);
            Ok(())
        }
        _ => Err(unsupported()),
    }
}

fn unsupported() -> Error {
    Error::invalid_geometry("only polygons, polygon meshes and triangular meshes can be merged")
}

fn merge_2d(leaves: &[&Euclidean2DGeometry]) -> Result<Geometry, Error> {
    let mut builder = Builder::<2>::default();
    let mut elevation: Option<Option<u64>> = None;
    let mut check_elevation = |z: Option<f64>| -> Result<(), Error> {
        let bits = z.map(f64::to_bits);
        match elevation {
            None => elevation = Some(bits),
            Some(seen) if seen != bits => {
                return Err(Error::invalid_geometry(
                    "cannot merge 2D geometries at different elevations",
                ));
            }
            Some(_) => {}
        }
        Ok(())
    };
    for leaf in leaves {
        match leaf {
            Euclidean2DGeometry::Polygon(p) => {
                builder.check_frame(p.frame())?;
                check_elevation(p.elevation())?;
                builder.push_polygon(
                    std::iter::once(p.exterior()).chain(p.interiors()),
                    p.appearance().as_ref(),
                );
            }
            Euclidean2DGeometry::PolygonMesh(m) => {
                builder.check_frame(m.frame())?;
                check_elevation(m.elevation())?;
                let (indices, offsets, interiors) = m.csr_buffers();
                builder.push_polygon_mesh(
                    m.vertices(),
                    indices,
                    offsets,
                    interiors,
                    m.appearance().as_ref(),
                );
            }
            Euclidean2DGeometry::TriangularMesh(m) => {
                builder.check_frame(m.frame())?;
                check_elevation(m.elevation())?;
                builder.push_triangular_mesh(m.vertices(), m.triangles(), m.appearance().as_ref());
            }
            _ => return Err(unsupported()),
        }
    }
    let z = elevation.flatten().map(f64::from_bits);
    let Some(frame) = builder.frame.clone() else {
        return Ok(Geometry::None);
    };
    if builder.face_ends.is_empty() {
        return Ok(Geometry::None);
    }
    let appearance = builder.appearance(|v| [v[0], v[1], z.unwrap_or(0.0)])?;
    let geometry = if builder.all_triangular {
        let mut mesh = TriangularMesh2D::from_parts(frame, builder.vertices, builder.face_indices)?;
        mesh.set_elevation(z);
        mesh.set_raw_appearance(appearance);
        Euclidean2DGeometry::TriangularMesh(Box::new(mesh))
    } else {
        let face_offsets = internal_boundaries(&builder.face_ends);
        let mut mesh = PolygonMesh2D::from_raw_parts(
            frame,
            builder.vertices,
            builder.face_indices,
            face_offsets,
            builder.interior_offsets,
        )?;
        mesh.set_elevation(z);
        *mesh.appearance_mut() = appearance;
        Euclidean2DGeometry::PolygonMesh(Box::new(mesh))
    };
    Ok(Geometry::Euclidean2D(geometry))
}

fn merge_3d(leaves: &[&Euclidean3DGeometry]) -> Result<Geometry, Error> {
    let mut builder = Builder::<3>::default();
    for leaf in leaves {
        match leaf {
            Euclidean3DGeometry::Polygon(p) => {
                builder.check_frame(p.frame())?;
                builder.push_polygon(
                    std::iter::once(p.exterior()).chain(p.interiors()),
                    p.appearance().as_ref(),
                );
            }
            Euclidean3DGeometry::PolygonMesh(m) => {
                builder.check_frame(m.frame())?;
                let (indices, offsets, interiors) = m.data().csr_buffers();
                builder.push_polygon_mesh(
                    m.vertices(),
                    indices,
                    offsets,
                    interiors,
                    m.appearance().as_ref(),
                );
            }
            Euclidean3DGeometry::TriangularMesh(m) => {
                builder.check_frame(m.frame())?;
                builder.push_triangular_mesh(m.vertices(), m.triangles(), m.appearance().as_ref());
            }
            _ => return Err(unsupported()),
        }
    }
    let Some(frame) = builder.frame.clone() else {
        return Ok(Geometry::None);
    };
    if builder.face_ends.is_empty() {
        return Ok(Geometry::None);
    }
    let appearance = builder.appearance(|v| v)?;
    let geometry = if builder.all_triangular {
        let mut mesh = TriangularMesh3D::from_parts(frame, builder.vertices, builder.face_indices)?;
        mesh.set_raw_appearance(appearance);
        Euclidean3DGeometry::TriangularMesh(Box::new(mesh))
    } else {
        let face_offsets = internal_boundaries(&builder.face_ends);
        let mut mesh = PolygonMesh3D::from_raw_parts(
            frame,
            builder.vertices,
            builder.face_indices,
            face_offsets,
            builder.interior_offsets,
        )?;
        *mesh.appearance_mut() = appearance;
        Euclidean3DGeometry::PolygonMesh(Box::new(mesh))
    };
    Ok(Geometry::Euclidean3D(geometry))
}

/// The face boundaries a CSR buffer stores: every face end but the last.
fn internal_boundaries(face_ends: &[u32]) -> Vec<u32> {
    face_ends[..face_ends.len().saturating_sub(1)].to_vec()
}

/// One input's run of faces and corners in the merged mesh.
struct Block<'a> {
    appearance: Option<&'a Appearance>,
    /// The input's face count.
    faces: usize,
    /// For each merged corner of this block, in order, the input's own corner
    /// index (the UV array index it reads from).
    source_corners: Vec<usize>,
}

/// The merged mesh being assembled.
struct Builder<'a, const N: usize> {
    frame: Option<CoordinateFrame>,
    vertices: Vec<[f64; N]>,
    weld: HashMap<[u64; N], u32>,
    face_indices: Vec<u32>,
    /// The end of every face in `face_indices`.
    face_ends: Vec<u32>,
    interior_offsets: Vec<u32>,
    all_triangular: bool,
    blocks: Vec<Block<'a>>,
}

impl<const N: usize> Default for Builder<'_, N> {
    fn default() -> Self {
        Self {
            frame: None,
            vertices: Vec::new(),
            weld: HashMap::new(),
            face_indices: Vec::new(),
            face_ends: Vec::new(),
            interior_offsets: Vec::new(),
            all_triangular: true,
            blocks: Vec::new(),
        }
    }
}

impl<'a, const N: usize> Builder<'a, N> {
    fn check_frame(&mut self, frame: &CoordinateFrame) -> Result<(), Error> {
        match &self.frame {
            None => self.frame = Some(frame.clone()),
            Some(seen) if seen != frame => {
                return Err(Error::invalid_geometry(
                    "cannot merge geometries in different coordinate frames",
                ));
            }
            Some(_) => {}
        }
        Ok(())
    }

    fn weld(&mut self, coord: [f64; N]) -> u32 {
        *self.weld.entry(coord.map(f64::to_bits)).or_insert_with(|| {
            self.vertices.push(coord);
            (self.vertices.len() - 1) as u32
        })
    }

    /// Append one polygon as one face. Each ring is stored closed; a closing
    /// corner added here reads the UV of the ring's first corner.
    fn push_polygon<'r>(
        &mut self,
        rings: impl Iterator<Item = &'r [[f64; N]]>,
        appearance: Option<&'a Appearance>,
    ) {
        self.all_triangular = false;
        let start = self.face_indices.len();
        let mut source_corners = Vec::new();
        let mut source = 0usize;
        for (i, ring) in rings.enumerate() {
            if ring.is_empty() {
                continue;
            }
            if i > 0 && self.face_indices.len() > start {
                self.interior_offsets.push(self.face_indices.len() as u32);
            } else if i > 0 {
                // A hole after an empty exterior has no face to belong to.
                source += ring.len();
                continue;
            }
            for (k, &coord) in ring.iter().enumerate() {
                let index = self.weld(coord);
                self.face_indices.push(index);
                source_corners.push(source + k);
            }
            if ring.len() >= 2 && ring.first() != ring.last() {
                let index = self.weld(ring[0]);
                self.face_indices.push(index);
                source_corners.push(source);
            }
            source += ring.len();
        }
        if self.face_indices.len() == start {
            return;
        }
        self.face_ends.push(self.face_indices.len() as u32);
        self.blocks.push(Block {
            appearance,
            faces: 1,
            source_corners,
        });
    }

    /// Append every face of a polygon mesh, keeping its corner order.
    fn push_polygon_mesh(
        &mut self,
        vertices: &[[f64; N]],
        indices: &IndexBuffer<1>,
        offsets: &IndexBuffer<1>,
        interiors: &IndexBuffer<1>,
        appearance: Option<&'a Appearance>,
    ) {
        if indices.len() == 0 {
            return;
        }
        self.all_triangular = false;
        let remap: Vec<u32> = vertices.iter().map(|&v| self.weld(v)).collect();
        let base = self.face_indices.len() as u32;
        self.face_indices
            .extend(indices.iter_u32().map(|[i]| remap[i as usize]));
        self.face_ends
            .extend(offsets.iter_u32().map(|[o]| base + o));
        self.face_ends.push(self.face_indices.len() as u32);
        self.interior_offsets
            .extend(interiors.iter_u32().map(|[o]| base + o));
        self.blocks.push(Block {
            appearance,
            faces: offsets.len() + 1,
            source_corners: (0..indices.len()).collect(),
        });
    }

    /// Append every triangle of a triangular mesh as a three-corner face.
    fn push_triangular_mesh(
        &mut self,
        vertices: &[[f64; N]],
        triangles: impl Iterator<Item = [u32; 3]>,
        appearance: Option<&'a Appearance>,
    ) {
        let remap: Vec<u32> = vertices.iter().map(|&v| self.weld(v)).collect();
        let start = self.face_indices.len();
        let mut faces = 0usize;
        for triangle in triangles {
            self.face_indices
                .extend(triangle.map(|i| remap[i as usize]));
            self.face_ends.push(self.face_indices.len() as u32);
            faces += 1;
        }
        if faces == 0 {
            return;
        }
        self.blocks.push(Block {
            appearance,
            faces,
            source_corners: (0..self.face_indices.len() - start).collect(),
        });
    }

    /// The merged appearance, or `None` when no input carries one.
    /// `position` lifts a vertex to the 3D point a `WorldToTexture` matrix is
    /// baked at.
    fn appearance(
        &self,
        position: impl Fn([f64; N]) -> [f64; 3],
    ) -> Result<Option<Appearance>, Error> {
        if self.blocks.iter().all(|b| b.appearance.is_none()) {
            return Ok(None);
        }

        let mut materials: Vec<Material> = Vec::new();
        let mut offsets: Vec<u32> = Vec::with_capacity(self.blocks.len());
        for block in &self.blocks {
            offsets.push(
                u32::try_from(materials.len())
                    .map_err(|_| Error::invalid_appearance("material palette too large"))?,
            );
            if let Some(app) = block.appearance {
                materials.extend(app.materials().iter().cloned());
            }
        }

        let mut theme_order: Vec<ThemeId> = Vec::new();
        for app in self.blocks.iter().filter_map(|b| b.appearance) {
            for binding in app.themes() {
                if !theme_order.contains(&binding.theme) {
                    theme_order.push(binding.theme.clone());
                }
            }
        }
        let default_theme = self
            .blocks
            .iter()
            .find_map(|b| b.appearance)
            .map(|app| app.default_theme().clone())
            .expect("an input carries appearance");

        let binding_of = |block: &Block<'a>, theme: &ThemeId| -> Option<&'a ThemeBinding> {
            block
                .appearance?
                .themes()
                .iter()
                .find(|b| &b.theme == theme)
        };

        let face_binding = |theme: &ThemeId, side: Side| -> Result<FaceBinding, Error> {
            let mut per_face: Vec<Option<MaterialIndex>> = Vec::with_capacity(self.face_ends.len());
            for (block, &offset) in self.blocks.iter().zip(&offsets) {
                let binding = binding_of(block, theme).and_then(|b| match side {
                    Side::Front => Some(&b.front),
                    Side::Back => b.back.as_ref(),
                });
                let Some(binding) = binding else {
                    per_face.resize(per_face.len() + block.faces, None);
                    continue;
                };
                match binding.clone().offset(offset)? {
                    FaceBinding::Uniform(index) => {
                        per_face.resize(per_face.len() + block.faces, Some(index));
                    }
                    FaceBinding::PerFace(faces) => {
                        per_face.extend((0..block.faces).map(|f| faces.get(f).copied().flatten()));
                    }
                }
            }
            Ok(FaceBinding::PerFace(per_face))
        };

        let total_corners = self.face_indices.len();
        let uv = |theme: &ThemeId, side: Side, channel: ChannelId| -> UvSource {
            let mut data: Vec<[f64; 2]> = Vec::with_capacity(total_corners);
            for block in &self.blocks {
                let source = binding_of(block, theme).and_then(|b| {
                    b.uv_sets
                        .iter()
                        .find(|s| s.side == side && s.channel == channel)
                        .map(|s| &s.uv)
                });
                let start = data.len();
                match source {
                    None => data.resize(start + block.source_corners.len(), [0.0, 0.0]),
                    Some(UvSource::Explicit(arr)) => data.extend(
                        block
                            .source_corners
                            .iter()
                            .map(|&c| arr.get(c).copied().unwrap_or([0.0, 0.0])),
                    ),
                    Some(UvSource::WorldToTexture(matrix)) => {
                        data.extend((0..block.source_corners.len()).map(|k| {
                            let vertex = self.vertices[self.face_indices[start + k] as usize];
                            bake(matrix, position(vertex))
                        }))
                    }
                }
            }
            UvSource::Explicit(data.into_boxed_slice())
        };

        let mut themes = Vec::with_capacity(theme_order.len());
        for theme in &theme_order {
            let front = face_binding(theme, Side::Front)?;
            let has_back = self
                .blocks
                .iter()
                .any(|b| binding_of(b, theme).is_some_and(|b| b.back.is_some()));
            let back = has_back
                .then(|| face_binding(theme, Side::Back))
                .transpose()?;
            let mut keys: Vec<(Side, ChannelId)> = Vec::new();
            let mut seen: HashSet<(Side, ChannelId)> = HashSet::new();
            for block in &self.blocks {
                for set in binding_of(block, theme).map_or(&[][..], |b| &b.uv_sets) {
                    if seen.insert((set.side, set.channel)) {
                        keys.push((set.side, set.channel));
                    }
                }
            }
            let uv_sets = keys
                .into_iter()
                .map(|(side, channel)| UvSet {
                    side,
                    channel,
                    uv: uv(theme, side, channel),
                })
                .collect();
            themes.push(ThemeBinding {
                theme: theme.clone(),
                front,
                back,
                uv_sets,
            });
        }

        let mut appearance = Appearance::from_parts(materials, themes, default_theme);
        appearance.compact_materials();
        Ok(Some(appearance))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection3D;
    use crate::coordinate::EpsgCode;
    use crate::point::Point3D;
    use crate::polygon::{Polygon2D, Polygon3D};
    use crate::test_support::*;

    const F: CoordinateFrame = CoordinateFrame::Euclidean;

    fn tri(corners: [[f64; 3]; 3]) -> Polygon3D {
        Polygon3D::from_rings(F, corners, Vec::<Vec<[f64; 3]>>::new())
    }

    fn g3(geometry: Euclidean3DGeometry) -> Geometry {
        Geometry::Euclidean3D(geometry)
    }

    fn polygon(p: Polygon3D) -> Geometry {
        g3(Euclidean3DGeometry::Polygon(Box::new(p)))
    }

    fn trimesh(vertices: Vec<[f64; 3]>, indices: Vec<u32>) -> Geometry {
        g3(Euclidean3DGeometry::TriangularMesh(Box::new(
            TriangularMesh3D::from_parts(F, vertices, indices).unwrap(),
        )))
    }

    fn as_polygon_mesh(g: &Geometry) -> &PolygonMesh3D {
        let Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(m)) = g else {
            panic!("expected a polygon mesh, got {g:?}");
        };
        m
    }

    #[test]
    fn polygons_weld_into_one_polygon_mesh() {
        let a = tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
        let b = tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        let merged = merge_meshes(&[polygon(a), polygon(b)]).unwrap();
        let mesh = as_polygon_mesh(&merged);
        assert_eq!(mesh.num_faces(), 2);
        assert_eq!(mesh.vertices().len(), 4);
        assert!(mesh.appearance().is_none());
    }

    #[test]
    fn triangular_meshes_stay_triangular() {
        let a = trimesh(
            vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            vec![0, 1, 2],
        );
        let b = trimesh(
            vec![[1., 1., 0.], [0., 1., 0.], [1., 0., 0.]],
            vec![2, 0, 1],
        );
        let merged = merge_meshes(&[a, b]).unwrap();
        let Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(mesh)) = &merged else {
            panic!("expected a triangular mesh, got {merged:?}");
        };
        assert_eq!(mesh.vertices().len(), 4);
        assert_eq!(
            mesh.triangles().collect::<Vec<_>>(),
            vec![[0, 1, 2], [1, 3, 2]]
        );
    }

    #[test]
    fn a_triangular_mesh_with_a_polygon_makes_a_polygon_mesh() {
        let a = trimesh(
            vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            vec![0, 1, 2],
        );
        let b = polygon(tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]));
        let merged = merge_meshes(&[a, b]).unwrap();
        let mesh = as_polygon_mesh(&merged);
        assert_eq!(mesh.num_faces(), 2);
        assert_eq!(mesh.vertices().len(), 4);
    }

    #[test]
    fn collections_are_flattened() {
        let a =
            Euclidean3DGeometry::Polygon(Box::new(tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]])));
        let b =
            Euclidean3DGeometry::Polygon(Box::new(tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]])));
        let collection = g3(Euclidean3DGeometry::Collection(Collection3D::new([a, b])));
        let merged = merge_meshes(&[collection]).unwrap();
        assert_eq!(as_polygon_mesh(&merged).num_faces(), 2);
    }

    #[test]
    fn a_hole_is_kept() {
        let outer = [[0., 0., 0.], [4., 0., 0.], [4., 4., 0.], [0., 4., 0.]];
        let hole = vec![[1., 1., 0.], [2., 1., 0.], [2., 2., 0.], [1., 2., 0.]];
        let with_hole = Polygon3D::from_rings(F, outer, vec![hole]);
        let merged = merge_meshes(&[polygon(with_hole.clone())]).unwrap();
        let faces = as_polygon_mesh(&merged).faces();
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].interiors().count(), 1);
    }

    #[test]
    fn appearance_is_carried_per_face() {
        let mut a = tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
        a.set_appearance(
            theme("rgb"),
            textured(),
            Some(explicit_uv(&[[0., 0.], [1., 0.], [0., 1.]])),
        )
        .unwrap();
        let b = tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        let merged = merge_meshes(&[polygon(a), polygon(b)]).unwrap();
        let mesh = as_polygon_mesh(&merged);
        let app = mesh.appearance().as_ref().unwrap();
        assert_eq!(app.materials().len(), 1);
        let binding = &app.themes()[0];
        assert_eq!(
            binding.front,
            FaceBinding::PerFace(vec![MaterialIndex::new(0), None])
        );
        let UvSource::Explicit(uv) = &binding.uv_sets[0].uv else {
            panic!("expected explicit UV");
        };
        // Each triangle is stored closed: four corners per face.
        assert_eq!(
            uv.to_vec(),
            vec![
                [0., 0.],
                [1., 0.],
                [0., 1.],
                [0., 0.],
                [0., 0.],
                [0., 0.],
                [0., 0.],
                [0., 0.]
            ]
        );
    }

    #[test]
    fn mesh_appearance_is_offset_into_the_merged_palette() {
        let mut a = tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
        a.set_appearance(theme("rgb"), phong(None), None).unwrap();
        let mut b = tri([[1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
        b.set_appearance(theme("rgb"), textured(), Some(uv(3)))
            .unwrap();
        let merged = merge_meshes(&[polygon(a), polygon(b)]).unwrap();
        let app = as_polygon_mesh(&merged).appearance().clone().unwrap();
        assert_eq!(app.materials().len(), 2);
        assert_eq!(
            app.themes()[0].front,
            FaceBinding::PerFace(vec![MaterialIndex::new(0), MaterialIndex::new(1)])
        );
    }

    #[test]
    fn mismatches_are_errors() {
        let a = polygon(tri([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]));
        let other_frame = polygon(Polygon3D::from_rings(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            Vec::<Vec<[f64; 3]>>::new(),
        ));
        assert!(merge_meshes(&[a.clone(), other_frame]).is_err());

        let flat = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                F,
                [[0., 0.], [1., 0.], [0., 1.]],
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )));
        assert!(merge_meshes(&[a.clone(), flat]).is_err());

        let point = g3(Euclidean3DGeometry::Point(Point3D::new(F, [0., 0., 0.])));
        assert!(!is_mergeable(&point));
        assert!(merge_meshes(&[a, point]).is_err());
    }

    #[test]
    fn two_dimensional_elevations_must_agree() {
        let at = |z: f64| {
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
                Polygon2D::from_rings_at_elevation(
                    F,
                    [[0., 0.], [1., 0.], [0., 1.]],
                    Vec::<Vec<[f64; 2]>>::new(),
                    z,
                ),
            )))
        };
        let Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(mesh)) =
            merge_meshes(&[at(5.0), at(5.0)]).unwrap()
        else {
            panic!("expected a 2D polygon mesh");
        };
        assert_eq!(mesh.elevation(), Some(5.0));
        assert!(merge_meshes(&[at(5.0), at(6.0)]).is_err());
    }

    #[test]
    fn nothing_to_merge_yields_none() {
        assert_eq!(merge_meshes(&[Geometry::None]).unwrap(), Geometry::None);
    }
}
