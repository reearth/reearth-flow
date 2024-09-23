use crate::{
    algorithm::{coordinate_position::CoordPos, GeoFloat},
    types::coordinate::Coordinate,
};

use super::{Direction, Edge, EdgeEnd, GeometryGraph, IntersectionMatrix, Label};

/// A collection of [`EdgeEnds`](EdgeEnd) which obey the following invariant:
/// They originate at the same node and have the same direction.
///
/// This is based on [JTS's `EdgeEndBundle` as of 1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/operation/relate/EdgeEndBundle.java)
#[derive(Clone, Debug)]
pub(crate) struct EdgeEndBundle<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    coordinate: Coordinate<T, Z>,
    edge_ends: Vec<EdgeEnd<T, Z>>,
}

impl<T, Z> EdgeEndBundle<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    pub(crate) fn new(coordinate: Coordinate<T, Z>) -> Self {
        Self {
            coordinate,
            edge_ends: vec![],
        }
    }

    fn edge_ends_iter(&self) -> impl Iterator<Item = &EdgeEnd<T, Z>> {
        self.edge_ends.iter()
    }

    fn edge_ends_iter_mut(&mut self) -> impl Iterator<Item = &mut EdgeEnd<T, Z>> {
        self.edge_ends.iter_mut()
    }

    pub(crate) fn insert(&mut self, edge_end: EdgeEnd<T, Z>) {
        self.edge_ends.push(edge_end);
    }

    pub(crate) fn into_labeled(mut self) -> LabeledEdgeEndBundle<T, Z> {
        let is_area = self
            .edge_ends_iter()
            .any(|edge_end| edge_end.label().is_area());

        let mut label = if is_area {
            Label::empty_area()
        } else {
            Label::empty_line_or_point()
        };

        for i in 0..2 {
            self.compute_label_on(&mut label, i);
            if is_area {
                self.compute_label_side(&mut label, i, Direction::Left);
                self.compute_label_side(&mut label, i, Direction::Right);
            }
        }

        LabeledEdgeEndBundle {
            label,
            edge_end_bundle: self,
        }
    }

    fn compute_label_on(&mut self, label: &mut Label, geom_index: usize) {
        let mut boundary_count = 0;
        let mut found_interior = false;

        for edge_end in self.edge_ends_iter() {
            match edge_end.label().on_position(geom_index) {
                Some(CoordPos::OnBoundary) => {
                    boundary_count += 1;
                }
                Some(CoordPos::Inside) => {
                    found_interior = true;
                }
                None | Some(CoordPos::Outside) => {}
            }
        }

        let mut position = None;
        if found_interior {
            position = Some(CoordPos::Inside);
        }

        if boundary_count > 0 {
            position = Some(GeometryGraph::<'_, T, Z>::determine_boundary(
                boundary_count,
            ));
        }

        if let Some(location) = position {
            label.set_on_position(geom_index, location);
        } else {
            // This is technically a diversion from JTS, but I don't think we'd ever
            // get here, unless `l.on_location` was *already* None, in which cases this is a
            // no-op, so assert that assumption.
            // If this assert is rightfully triggered, we may need to add a method like
            // `l.clear_on_location(geom_index)`
            debug_assert!(
                label.on_position(geom_index).is_none(),
                "diverging from JTS, which would have replaced the existing Location with None"
            );
        }
    }

    /// To compute the summary label for a side, the algorithm is:
    ///     FOR all edges
    ///       IF any edge's location is INTERIOR for the side, side location = INTERIOR
    ///       ELSE IF there is at least one EXTERIOR attribute, side location = EXTERIOR
    ///       ELSE  side location = NULL
    /// Note that it is possible for two sides to have apparently contradictory information
    /// i.e. one edge side may indicate that it is in the interior of a geometry, while
    /// another edge side may indicate the exterior of the same geometry.  This is
    /// not an incompatibility - GeometryCollections may contain two Polygons that touch
    /// along an edge.  This is the reason for Interior-primacy rule above - it
    /// results in the summary label having the Geometry interior on _both_ sides.
    fn compute_label_side(&mut self, label: &mut Label, geom_index: usize, side: Direction) {
        let mut position = None;
        for edge_end in self.edge_ends_iter_mut() {
            if edge_end.label().is_area() {
                match edge_end.label_mut().position(geom_index, side) {
                    Some(CoordPos::Inside) => {
                        position = Some(CoordPos::Inside);
                        break;
                    }
                    Some(CoordPos::Outside) => {
                        position = Some(CoordPos::Outside);
                    }
                    None | Some(CoordPos::OnBoundary) => {}
                }
            }
        }

        if let Some(position) = position {
            label.set_position(geom_index, side, position);
        }
    }
}

/// An [`EdgeEndBundle`] whose topological relationships have been aggregated into a single
/// [`Label`].
///
/// `update_intersection_matrix` applies this aggregated topology to an `IntersectionMatrix`.
#[derive(Clone, Debug)]
pub(crate) struct LabeledEdgeEndBundle<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    label: Label,
    edge_end_bundle: EdgeEndBundle<T, Z>,
}

impl<T, Z> LabeledEdgeEndBundle<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }

    pub fn update_intersection_matrix(&self, intersection_matrix: &mut IntersectionMatrix) {
        Edge::<T, Z>::update_intersection_matrix(self.label(), intersection_matrix);
    }

    pub fn coordinate(&self) -> &Coordinate<T, Z> {
        &self.edge_end_bundle.coordinate
    }
}
