use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};

use crate::{
    algorithm::utils::{denormalize_vertices, normalize_vertices},
    types::{
        coordinate::Coordinate,
        coordnum::CoordNum,
        solid::{Solid, Solid3D},
        triangle::Triangle,
    },
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
        let mut right = right.as_triangle_mesh()?;
        let mut left = left.as_triangle_mesh()?;
        let mut union = left.clone().union(right.clone())?;
        let norm = normalize_vertices(union.get_vertices_mut());
        right
            .get_vertices_mut()
            .iter_mut()
            .for_each(|v| *v = (*v - norm.translation) / norm.scale);
        left.get_vertices_mut()
            .iter_mut()
            .for_each(|v| *v = (*v - norm.translation) / norm.scale);
        let two_manifolds = union.into_2_manifolds_with_boundaries();
        let mut result_faces = Vec::new();
        // quick rejection for the intersection
        if two_manifolds.len() == 2 && self.operation == CSGOperation::Intersection {
            return Ok(Solid3D::new_with_faces(vec![]));
        }
        for mut two_manifold in two_manifolds {
            if two_manifold.is_empty() {
                continue;
            }
            let t = two_manifold
                .iter()
                .map(|t| {
                    (
                        t,
                        Triangle::new(
                            union.get_vertices()[t[0]],
                            union.get_vertices()[t[1]],
                            union.get_vertices()[t[2]],
                        )
                        .area(),
                    )
                })
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap()
                .0;
            let t = [
                union.get_vertices()[t[0]],
                union.get_vertices()[t[1]],
                union.get_vertices()[t[2]],
            ];
            let center = (t[0] + t[1] + t[2]) / 3_f64;

            let mut from_left = two_manifold
                .iter()
                .flatten()
                .map(|&v| union.get_vertices()[v])
                .all(|v| {
                    left.get_triangles().iter().any(|&w| {
                        let tri = Triangle::new(
                            left.get_vertices()[w[0]],
                            left.get_vertices()[w[1]],
                            left.get_vertices()[w[2]],
                        );
                        tri.contains(&v) || tri.boundary_contains(&v)
                    })
                });

            let mut from_right = two_manifold
                .iter()
                .flatten()
                .map(|&v| union.get_vertices()[v])
                .all(|v| {
                    right.get_triangles().iter().any(|&w| {
                        let tri = Triangle::new(
                            right.get_vertices()[w[0]],
                            right.get_vertices()[w[1]],
                            right.get_vertices()[w[2]],
                        );
                        tri.contains(&v) || tri.boundary_contains(&v)
                    })
                });

            if !from_left && !from_right {
                return Err("Triangle vertices are not all in one of the solids".into());
            } else if from_left && from_right {
                let mut left_vertices = two_manifold.iter().flatten().copied().collect::<Vec<_>>();
                left_vertices.sort_unstable();
                left_vertices.dedup();
                let num_coincidence_left = left_vertices
                    .iter()
                    .filter(|&&v| {
                        let v = union.get_vertices()[v];
                        left.get_vertices().iter().any(|&w| (w - v).norm() < 1e-8)
                    })
                    .count();
                let mut right_vertices = two_manifold.iter().flatten().copied().collect::<Vec<_>>();
                right_vertices.sort_unstable();
                right_vertices.dedup();
                let num_coincidence_right = right_vertices
                    .iter()
                    .filter(|&&v| {
                        let v = union.get_vertices()[v];
                        right.get_vertices().iter().any(|&w| (w - v).norm() < 1e-8)
                    })
                    .count();
                if num_coincidence_left > num_coincidence_right {
                    from_left = true;
                    from_right = false;
                } else {
                    from_left = false;
                    from_right = true;
                };
            };
            match self.operation {
                CSGOperation::Union => {
                    if !from_left && right.contains(&center) || left.contains(&center) && from_left
                    {
                        result_faces.append(&mut two_manifold);
                    }
                }
                CSGOperation::Intersection => {
                    if from_left && right.bounding_solid_contains(&center)
                        || left.bounding_solid_contains(&center) && from_right
                    {
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
        denormalize_vertices(union.get_vertices_mut(), norm);
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
