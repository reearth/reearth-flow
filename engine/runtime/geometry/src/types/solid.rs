use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};

use crate::types::coordinate::Coordinate;
use crate::types::line_string::LineString3D;
use crate::types::triangular_mesh::TriangularMesh;

use super::coordnum::CoordNum;
use super::face::Face;
use super::no_value::NoValue;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Default)]
pub struct Solid<T: CoordNum = f64, Z: CoordNum = f64> {
    boundary_surface: BoudarySurface<T, Z>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
pub enum BoudarySurface<T: CoordNum = f64, Z: CoordNum = f64> {
    Faces(Vec<Face<T, Z>>),
    TriangularMesh(TriangularMesh<T, Z>),
}

pub type Solid2D<T> = Solid<T, NoValue>;
pub type Solid3D<T> = Solid<T, T>;

impl<T: CoordNum, Z: CoordNum> Default for BoudarySurface<T, Z> {
    fn default() -> Self {
        BoudarySurface::Faces(vec![])
    }
}   

impl<T: CoordNum, Z: CoordNum> Solid<T, Z> {
    pub fn new_with_faces(faces: Vec<Face<T, Z>>) -> Self {
        Self { boundary_surface: BoudarySurface::Faces(faces) }
    }

    pub fn new_with_triangular_mesh(mesh: TriangularMesh<T, Z>) -> Self {
        Self { boundary_surface: BoudarySurface::TriangularMesh(mesh) }
    }

    pub fn all_faces(&self) -> Vec<Face<T, Z>> {
        match self.boundary_surface {
            BoudarySurface::Faces(ref faces) => faces.clone(),
            BoudarySurface::TriangularMesh(ref mesh) => 
                mesh.get_triangles().iter()
                    .map(|t| Face::<T, Z>::new(vec![
                        mesh.get_vertices()[t[0]], mesh.get_vertices()[t[1]], mesh.get_vertices()[t[2]], mesh.get_vertices()[t[0]]
                    ]))
                    .collect(),
        }
    }

    pub fn get_all_vertex_coordinates(&self) -> Vec<Coordinate<T, Z>> {
        match &self.boundary_surface {
            BoudarySurface::Faces(faces) => faces
                .iter()
                .flat_map(|f| f.0.iter().copied())
                .collect(),
            BoudarySurface::TriangularMesh(mesh) => mesh.get_vertices().to_vec(),
        }
    }
}

impl Solid3D<f64> {
    pub fn elevation(&self) -> f64 {
        match &self.boundary_surface {
            BoudarySurface::Faces(faces) => faces[0].0.first().map(|c| c.z).unwrap_or(0.0),
            BoudarySurface::TriangularMesh(mesh) => mesh
                .get_vertices()
                .first()
                .map(|c| c.z)
                .unwrap_or(0.0),
        }
    }

    pub fn is_elevation_zero(&self) -> bool {
        self.elevation() == 0.0
    }

    pub fn as_triangle_mesh(self) -> TriangularMesh<f64> {
        match self.boundary_surface {
            BoudarySurface::Faces(faces) => {
                let faces = faces.into_iter().map(|f| f.into()).collect::<Vec<LineString3D<f64>>>();
                TriangularMesh::from_faces(&faces)
            },
            BoudarySurface::TriangularMesh(mesh) => mesh,
        }
    }
}

impl From<Solid3D<f64>> for Solid2D<f64> {
    fn from(_: Solid3D<f64>) -> Solid2D<f64> {
        unreachable!("No 2D solid can be constructed")
    }
}

impl Solid3D<f64> {
    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        match &mut self.boundary_surface {
            BoudarySurface::Faces(faces) => faces.iter_mut().for_each(|f| f.transform_inplace(jgd2wgs)),
            BoudarySurface::TriangularMesh(mesh) => mesh.transform_inplace(jgd2wgs),
        }
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        match &mut self.boundary_surface {
            BoudarySurface::Faces(faces) => faces.iter_mut().for_each(|f| f.transform_offset(x, y, z)),
            BoudarySurface::TriangularMesh(mesh) => mesh.transform_offset(x, y, z),
        }
    }
}
