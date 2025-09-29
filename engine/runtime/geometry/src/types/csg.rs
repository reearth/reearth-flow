use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};

use crate::types::{coordinate::Coordinate, coordnum::CoordNum, solid::{Solid, Solid3D}, triangular_mesh::TriangularMesh};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CSG<T: CoordNum = f64, Z: CoordNum = f64> {
    left: CSGChild<T, Z>,
    right: CSGChild<T, Z>,
    operation: CSGOperation,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CSGChild<T: CoordNum = f64, Z: CoordNum = f64> {
    Solid(Solid<T, Z>),
    CSG(Box<CSG<T, Z>>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CSGOperation {
    Union,
    Intersection,
    Difference,
}

impl<T: CoordNum, Z: CoordNum> CSG<T, Z> {
    pub fn new(left: CSGChild<T, Z>, right: CSGChild<T, Z>, operation: CSGOperation) -> Self {
        Self {
            left,
            right,
            operation,
        }
    }

    pub fn left(&self) -> &CSGChild<T, Z> {
        &self.left
    }

    pub fn right(&self) -> &CSGChild<T, Z> {
        &self.right
    }

    pub fn operation(&self) -> &CSGOperation {
        &self.operation
    }

    pub fn get_all_vertex_coordinates(&self) -> Vec<Coordinate<T, Z>> {
        let mut coords = self.left.get_all_vertex_coordinates();
        coords.append(&mut self.right.get_all_vertex_coordinates());
        coords
    }
}

impl CSG<f64, f64> {
    pub fn elevation(&self) -> f64 {
        self.left.elevation().max(self.right.elevation())
    }

    pub fn is_elevation_zero(&self) -> bool {
        self.left.is_elevation_zero() && self.right.is_elevation_zero()
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.left.transform_offset(x, y, z);
        self.right.transform_offset(x, y, z);
    }

    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        self.left.transform_inplace(jgd2wgs);
        self.right.transform_inplace(jgd2wgs);
    }

    pub fn evaluate(self) -> Solid3D<f64> {
        let right = self.right.evaluate();
        let left = self.left.evaluate();
        match self.operation {
            CSGOperation::Union => left.union(&right),
            CSGOperation::Intersection => left.intersection(&right),
            CSGOperation::Difference => left.difference(&right),
            _ => unreachable!(),
        }
    }

}

impl<T: CoordNum, Z: CoordNum> CSGChild<T, Z> {
    pub fn get_all_vertex_coordinates(&self) -> Vec<Coordinate<T, Z>> {
        match self {
            CSGChild::Solid(geom) => geom.get_all_vertex_coordinates(),
            CSGChild::CSG(csg) => csg.get_all_vertex_coordinates(),
        }
    }
}

impl CSGChild<f64, f64> {
    pub fn elevation(&self) -> f64 {
        match self {
            CSGChild::Solid(geom) => geom.elevation(),
            CSGChild::CSG(csg) => csg.elevation(),
        }
    }
    
    pub fn is_elevation_zero(&self) -> bool {
        match self {
            CSGChild::Solid(geom) => geom.is_elevation_zero(),
            CSGChild::CSG(csg) => csg.is_elevation_zero(),
        }
    }
    
    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        match self {
            CSGChild::Solid(geom) => geom.transform_offset(x, y, z),
            CSGChild::CSG(csg) => {
                csg.left.transform_offset(x, y, z);
                csg.right.transform_offset(x, y, z);
            }
        }
    }
    
    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        match self {
            CSGChild::Solid(geom) => geom.transform_inplace(jgd2wgs),
            CSGChild::CSG(csg) => csg.transform_inplace(jgd2wgs),
        }
    }
}


/// takes in two solids and returns a triangular mesh representing the union of their surfaces.
/// This is a crutial step in evaluating a CSG operation, as it is a union of intesrsection and union operations.
fn surface_union(s1: &Solid3D<f64>, s2: &Solid3D<f64>) -> TriangularMesh<f64> {
    let s1 = TriangularMesh::from_faces(s1.all_faces());
    let s2 = TriangularMesh::from_faces(s2.all_faces());
    s1.union(&s2)
}