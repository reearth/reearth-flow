use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};

use crate::{
    types::{
        coordinate::Coordinate,
        coordnum::CoordNum,
        solid::{Solid, Solid3D},
    },
    utils::circumcenter,
};

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

    pub fn evaluate(self) -> Result<Solid3D<f64>, String> {
        let right = self.right.evaluate()?;
        let left = self.left.evaluate()?;
        println!("right: {:?}", right);
        println!("left: {:?}", left);
        let right = right.as_triangle_mesh();
        let left = left.as_triangle_mesh();
        let mut union = left.clone().union(right.clone())?;
        println!("union: {:?}", union);
        let two_manifolds = union.into_2_manifolds_with_boundaries();
        println!("two_manifolds.len: {:?}", two_manifolds);
        let mut result_faces = Vec::new();
        for mut two_manifold in two_manifolds {
            let t = two_manifold.first().ok_or("Failed to get first triangle")?;
            let t = [
                union.get_vertices().get(t[0]).ok_or("Failed to get vertex")?,
                union.get_vertices().get(t[1]).ok_or("Failed to get vertex")?,
                union.get_vertices().get(t[2]).ok_or("Failed to get vertex")?,
            ];
            println!("t: {:.2?}", t);
            let center = circumcenter(*t[0], *t[1], *t[2]).unwrap().0;
            match self.operation {
                CSGOperation::Union => {
                    if !left.bounding_solid_contains(&center) && right.contains(&center) {
                        result_faces.append(&mut two_manifold);
                    } else if left.contains(&center) && !right.bounding_solid_contains(&center) {
                        result_faces.append(&mut two_manifold);
                    }
                }
                CSGOperation::Intersection => {
                    if left.bounding_solid_contains(&center) && right.contains(&center) {
                        result_faces.append(&mut two_manifold);
                    } else if left.contains(&center) && right.bounding_solid_contains(&center) {
                        result_faces.append(&mut two_manifold);
                    }
                }
                CSGOperation::Difference => {
                    if left.bounding_solid_contains(&center) && right.contains(&center) {
                        result_faces.append(&mut two_manifold);
                    }
                }
            }
        }

        union.retain_faces(&result_faces);
        Ok(Solid3D::new_with_triangular_mesh(union))
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

    fn evaluate(self) -> Result<Solid3D<f64>, String> {
        match self {
            CSGChild::Solid(geom) => Ok(geom),
            CSGChild::CSG(csg) => csg.evaluate(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::triangular_mesh::tests::{create_simple_obj, get_cube};

    #[test]
    fn test_csg_evaluate_union() {
        let cube1 = get_cube();
        let mut cube2 = get_cube();
        cube2.transform_offset(-0.8, -0.7, -0.6);
        let solid1 = Solid3D::new_with_triangular_mesh(cube1);
        let solid2 = Solid3D::new_with_triangular_mesh(cube2);
        let csg = CSG::new(
            CSGChild::Solid(solid1),
            CSGChild::Solid(solid2),
            CSGOperation::Intersection,
        );
        let result = csg.evaluate().unwrap();
        let mesh = result.clone().as_triangle_mesh();
        create_simple_obj(&result.get_all_vertex_coordinates(), &mesh.get_triangles());
    }
}