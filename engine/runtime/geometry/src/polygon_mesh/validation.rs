use super::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};
use crate::coordinate::CoordinateFrame;
use crate::index::IndexBuffer;
use crate::validation_next::{
    check_duplicate_points_2d, check_duplicate_points_3d, check_finite_2d, check_finite_3d,
    check_too_few_points_2d, check_too_few_points_3d, check_unclosed_ring_2d,
    check_unclosed_ring_3d, Validate, ValidationReport, ValidationType,
};

/// Decode the CSR face topology into the flat vertex-index buffer plus one
/// `(start, end)` range per ring — each face's exterior ring, then its hole
/// rings. Ranges index the returned buffer.
fn ring_ranges(
    face_indices: &IndexBuffer<1>,
    face_offsets: &IndexBuffer<1>,
    interior_offsets: &IndexBuffer<1>,
) -> (Vec<u32>, Vec<(usize, usize)>) {
    let indices: Vec<u32> = face_indices.iter_u32().map(|[i]| i).collect();
    let n = indices.len();
    let mut ranges = Vec::new();
    if n == 0 {
        return (indices, ranges);
    }
    let face_ends: Vec<usize> = face_offsets.iter_u32().map(|[i]| i as usize).collect();
    let holes: Vec<usize> = interior_offsets.iter_u32().map(|[i]| i as usize).collect();
    let n_faces = face_ends.len() + 1;
    let mut start = 0usize;
    for f in 0..n_faces {
        let end = face_ends.get(f).copied().unwrap_or(n);
        // Hole rings of this face begin at the interior offsets inside (start, end).
        let mut ring_start = start;
        for &h in holes.iter().filter(|&&h| h > start && h < end) {
            ranges.push((ring_start, h));
            ring_start = h;
        }
        ranges.push((ring_start, end));
        start = end;
    }
    (indices, ranges)
}

impl PolygonMesh3DData {
    /// Report a [`ValidationType::TooFewPoints`] problem for every face ring with
    /// fewer than four (closed-ring) vertices. Shared by the [`PolygonMesh3D`]
    /// leaf and [`Solid`](crate::solid::Solid) shells, which supply the frame.
    pub(crate) fn check_too_few_points(
        &self,
        frame: &CoordinateFrame,
        report: &mut ValidationReport,
    ) {
        let (indices, ranges) = ring_ranges(
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
        );
        for (s, e) in ranges {
            if e - s < 4 {
                let coords: Vec<[f64; 3]> = indices[s..e]
                    .iter()
                    .map(|&i| self.vertices[i as usize])
                    .collect();
                check_too_few_points_3d(frame, &coords, true, report);
            }
        }
    }

    /// Report a [`ValidationType::UnclosedRing`] problem for every face ring whose
    /// first and last vertices differ. Shared by the [`PolygonMesh3D`] leaf and
    /// [`Solid`](crate::solid::Solid) shells, which supply the frame.
    pub(crate) fn check_unclosed_rings(
        &self,
        frame: &CoordinateFrame,
        report: &mut ValidationReport,
    ) {
        let (indices, ranges) = ring_ranges(
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
        );
        for (s, e) in ranges {
            // Only materialize the ring when its endpoints actually differ.
            if e > s && self.vertices[indices[s] as usize] != self.vertices[indices[e - 1] as usize]
            {
                let coords: Vec<[f64; 3]> = indices[s..e]
                    .iter()
                    .map(|&i| self.vertices[i as usize])
                    .collect();
                check_unclosed_ring_3d(frame, &coords, report);
            }
        }
    }
}

impl Validate for PolygonMesh2D {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_2d(&self.frame, &self.vertices, self.z.as_deref(), &mut report);
        match &valid_type {
            ValidationType::TooFewPoints => {
                let (indices, ranges) = ring_ranges(
                    &self.face_indices,
                    &self.face_offsets,
                    &self.interior_offsets,
                );
                for (s, e) in ranges {
                    if e - s < 4 {
                        let coords: Vec<[f64; 2]> = indices[s..e]
                            .iter()
                            .map(|&i| self.vertices[i as usize])
                            .collect();
                        check_too_few_points_2d(&self.frame, &coords, true, &mut report);
                    }
                }
            }
            ValidationType::UnclosedRing => {
                let (indices, ranges) = ring_ranges(
                    &self.face_indices,
                    &self.face_offsets,
                    &self.interior_offsets,
                );
                for (s, e) in ranges {
                    if e > s
                        && self.vertices[indices[s] as usize]
                            != self.vertices[indices[e - 1] as usize]
                    {
                        let coords: Vec<[f64; 2]> = indices[s..e]
                            .iter()
                            .map(|&i| self.vertices[i as usize])
                            .collect();
                        check_unclosed_ring_2d(&self.frame, &coords, &mut report);
                    }
                }
            }
            // A shared vertex pool should hold no coincident vertices; elevation
            // is not considered.
            ValidationType::DuplicatePoints { tolerance } => check_duplicate_points_2d(
                &self.frame,
                self.vertices.iter().copied(),
                *tolerance,
                &mut report,
            ),
            _ => {}
        }
        report.into_option()
    }
}

impl Validate for PolygonMesh3D {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_3d(
            &self.frame,
            self.data.vertices().iter().copied(),
            &mut report,
        );
        match &valid_type {
            ValidationType::TooFewPoints => {
                self.data.check_too_few_points(&self.frame, &mut report)
            }
            ValidationType::UnclosedRing => {
                self.data.check_unclosed_rings(&self.frame, &mut report)
            }
            ValidationType::DuplicatePoints { tolerance } => check_duplicate_points_3d(
                &self.frame,
                self.data.vertices().iter().copied(),
                *tolerance,
                &mut report,
            ),
            _ => {}
        }
        report.into_option()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polygon_mesh::PolygonMesh3D;

    fn quad_verts() -> Vec<[f64; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn closed_face_ring_is_valid() {
        // Ring stored closed (last index repeats the first).
        let m = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            quad_verts(),
            [[0u32, 1, 2, 3, 0]],
        )
        .unwrap();
        assert!(m.validate(ValidationType::UnclosedRing).is_none());
        assert!(m.validate(ValidationType::TooFewPoints).is_none());
    }

    #[test]
    fn open_face_ring_is_unclosed() {
        let m =
            PolygonMesh3D::from_parts(CoordinateFrame::Euclidean, quad_verts(), [[0u32, 1, 2, 3]])
                .unwrap();
        let report = m.validate(ValidationType::UnclosedRing).unwrap();
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.0[0].problem, "UnclosedRing");
    }

    #[test]
    fn short_face_ring_is_too_few_points() {
        // A closed but degenerate ring: [0,1,0] is only three stored coords.
        let m = PolygonMesh3D::from_parts(CoordinateFrame::Euclidean, quad_verts(), [[0u32, 1, 0]])
            .unwrap();
        let report = m.validate(ValidationType::TooFewPoints).unwrap();
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.0[0].problem, "TooFewPoints");
    }
}
