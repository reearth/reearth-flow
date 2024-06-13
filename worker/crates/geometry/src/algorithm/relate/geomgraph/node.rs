use crate::{
    algorithm::{coordinate_position::CoordPos, dimensions::Dimensions, GeoFloat},
    types::coordinate::Coordinate,
};

use super::{EdgeEnd, EdgeEndBundleStar, IntersectionMatrix, Label};

#[derive(Debug, Clone)]
pub(crate) struct CoordNode<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    coordinate: Coordinate<T, Z>,
    label: Label,
}

impl<T: GeoFloat, Z: GeoFloat> CoordNode<T, Z> {
    pub(crate) fn label(&self) -> &Label {
        &self.label
    }

    pub(crate) fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }

    pub(crate) fn is_isolated(&self) -> bool {
        self.label.geometry_count() == 1
    }
}

impl<T, Z> CoordNode<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    pub fn new(coordinate: Coordinate<T, Z>) -> CoordNode<T, Z> {
        CoordNode {
            coordinate,
            label: Label::empty_line_or_point(),
        }
    }

    pub fn coordinate(&self) -> &Coordinate<T, Z> {
        &self.coordinate
    }

    pub fn set_label_on_position(&mut self, geom_index: usize, position: CoordPos) {
        self.label.set_on_position(geom_index, position)
    }

    /// Updates the label of a node to BOUNDARY, obeying the mod-2 rule.
    pub fn set_label_boundary(&mut self, geom_index: usize) {
        let new_position = match self.label.on_position(geom_index) {
            Some(CoordPos::OnBoundary) => CoordPos::Inside,
            Some(CoordPos::Inside) => CoordPos::OnBoundary,
            None | Some(CoordPos::Outside) => CoordPos::OnBoundary,
        };
        self.label.set_on_position(geom_index, new_position);
    }

    pub fn update_intersection_matrix(&self, intersection_matrix: &mut IntersectionMatrix) {
        intersection_matrix.set_at_least_if_in_both(
            self.label.on_position(0),
            self.label.on_position(1),
            Dimensions::ZeroDimensional,
        );
    }
}
