use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};

use crate::types::coordinate::Coordinate;
use crate::types::triangular_mesh::TriangularMesh;

use super::coordnum::CoordNum;
use super::face::Face;
use super::no_value::NoValue;
use super::traits::Elevation;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Default)]
pub struct Solid<T: CoordNum = f64, Z: CoordNum = f64> {
    boundary_surface: BoudarySurface<T, Z>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
pub enum BoudarySurface<T: CoordNum = f64, Z: CoordNum = f64> {
    Faces(Vec<Face<T, Z>>),
    TriangularMesh(TriangularMesh<T>),
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

    pub fn new_with_triangular_mesh(mesh: TriangularMesh<T>) -> Self {
        Self { boundary_surface: BoudarySurface::TriangularMesh(mesh) }
    }

    pub fn all_faces(&self) -> Option<Vec<&Face<T, Z>>> {
        if let BoudarySurface::Faces(ref faces) = self.boundary_surface {
            Some(faces.iter().collect())
        } else {
            None
        }

    }

    pub fn get_all_vertex_coordinates(&self) -> Vec<Coordinate<T, Z>> {
        match &self.boundary_surface {
            BoudarySurface::Faces(faces) => faces
                .iter()
                .flat_map(|f| f.get_all_vertex_coordinates())
                .collect(),
            BoudarySurface::TriangularMesh(mesh) => mesh.get_all_vertex_coordinates(),
        }
    }
}

impl Solid3D<f64> {
    pub fn elevation(&self) -> f64 {
        self.top
            .first()
            .map(|t| t.0.first().map(|c| c.z.into()).unwrap_or(0_f64))
            .unwrap_or(0.0)
    }

    pub fn is_elevation_zero(&self) -> bool {
        self.bottom.iter().all(|f| f.is_elevation_zero())
            && self.top.iter().all(|f| f.is_elevation_zero())
            && self.sides.iter().all(|f| f.is_elevation_zero())
    }
}

impl From<Solid3D<f64>> for Solid2D<f64> {
    fn from(p: Solid3D<f64>) -> Solid2D<f64> {
        Solid2D::new(
            p.bottom.into_iter().map(|c| c.into()).collect(),
            p.top.into_iter().map(|c| c.into()).collect(),
            p.sides.into_iter().map(|c| c.into()).collect(),
        )
    }
}

impl Solid3D<f64> {
    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        self.bottom
            .iter_mut()
            .for_each(|f| f.transform_inplace(jgd2wgs));
        self.top
            .iter_mut()
            .for_each(|f| f.transform_inplace(jgd2wgs));
        self.sides
            .iter_mut()
            .for_each(|f| f.transform_inplace(jgd2wgs));
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.bottom
            .iter_mut()
            .for_each(|f| f.transform_offset(x, y, z));
        self.top
            .iter_mut()
            .for_each(|f| f.transform_offset(x, y, z));
        self.sides
            .iter_mut()
            .for_each(|f| f.transform_offset(x, y, z));
    }
}
