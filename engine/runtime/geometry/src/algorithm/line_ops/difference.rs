use crate::{
    algorithm::GeoFloat,
    types::{coordinate::Coordinate2D, line::Line2D},
};

pub fn line_difference_2d<T: GeoFloat>(
    line0: Line2D<T>,
    line1: Line2D<T>,
    tolerance: T,
) -> Vec<Line2D<T>> {
    // Linear interpolation
    fn lerp<T: GeoFloat>(a: Coordinate2D<T>, b: Coordinate2D<T>, t: T) -> Coordinate2D<T> {
        Coordinate2D::new_(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
    }

    // Get the relative position of the intersection point on the line
    fn t_of<T: GeoFloat>(line: &Line2D<T>, pt: &Coordinate2D<T>) -> T {
        let dx_line = line.end.x - line.start.x;
        let dy_line = line.end.y - line.start.y;
        let dx_pt = pt.x - line.start.x;
        let dy_pt = pt.y - line.start.y;

        let dot = dx_line * dx_pt + dy_line * dy_pt;
        let len_sq = dx_line * dx_line + dy_line * dy_line;

        dot / len_sq
    }

    // Check if the line segments are on the same straight line (determined by cross product)
    let cross = (line0.end.x - line0.start.x) * (line1.end.y - line1.start.y)
        - (line0.end.y - line0.start.y) * (line1.end.x - line1.start.x);

    if cross.abs() > tolerance {
        // No intersection (lines are not on the same straight line)
        return vec![line0];
    }

    let t_start = T::zero();
    let t_end = T::one();

    let mut t0 = t_of(&line0, &line1.start);
    let mut t1 = t_of(&line0, &line1.end);

    if t0 > t1 {
        std::mem::swap(&mut t0, &mut t1);
    }

    let overlap_start = t0.clamp(t_start, t_end);
    let overlap_end = t1.clamp(t_start, t_end);

    if overlap_start >= overlap_end {
        // No intersection
        return vec![line0];
    }

    let mut segments = vec![];

    if overlap_start > t_start {
        segments.push(Line2D {
            start: line0.start,
            end: lerp(line0.start, line0.end, overlap_start),
        });
    }

    if overlap_end < t_end {
        segments.push(Line2D {
            start: lerp(line0.start, line0.end, overlap_end),
            end: line0.end,
        });
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-6;

    #[test]
    fn test_no_overlap() {
        let line0 = Line2D::new((0.0, 0.0), (1.0, 1.0));
        let line1 = Line2D::new((2.0, 2.0), (3.0, 3.0));

        let result = line_difference_2d(line0, line1, EPSILON);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], line0);
    }

    #[test]
    fn test_full_overlap() {
        let line0 = Line2D::new((0.0, 0.0), (2.0, 2.0));
        let line1 = Line2D::new((0.0, 0.0), (2.0, 2.0));

        let result = line_difference_2d(line0, line1, EPSILON);
        assert!(result.is_empty());
    }

    #[test]
    fn test_partial_overlap_start() {
        let line0 = Line2D::new((0.0, 0.0), (2.0, 2.0));
        let line1 = Line2D::new((0.0, 0.0), (1.0, 1.0));

        let result = line_difference_2d(line0, line1, EPSILON);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Line2D::new((1.0, 1.0), (2.0, 2.0)));
    }

    #[test]
    fn test_partial_overlap_end() {
        let line0 = Line2D::new((0.0, 0.0), (2.0, 2.0));
        let line1 = Line2D::new((1.0, 1.0), (2.0, 2.0));

        let result = line_difference_2d(line0, line1, EPSILON);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Line2D::new((0.0, 0.0), (1.0, 1.0)));
    }

    #[test]
    fn test_partial_overlap_middle() {
        let line0 = Line2D::new((0.0, 0.0), (3.0, 3.0));
        let line1 = Line2D::new((1.0, 1.0), (2.0, 2.0));

        let result = line_difference_2d(line0, line1, EPSILON);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Line2D::new((0.0, 0.0), (1.0, 1.0)));
        assert_eq!(result[1], Line2D::new((2.0, 2.0), (3.0, 3.0)));
    }

    #[test]
    fn test_overlap_outside() {
        let line0 = Line2D::new((1.0, 1.0), (2.0, 2.0));
        let line1 = Line2D::new((0.0, 0.0), (3.0, 3.0));

        let result = line_difference_2d(line0, line1, EPSILON);
        println!("{:?}", result);
        assert!(result.is_empty());
    }

    #[test]
    fn test_reverse_direction() {
        let line0 = Line2D::new((0.0, 0.0), (3.0, 3.0));
        let line1 = Line2D::new((2.0, 2.0), (1.0, 1.0));

        let result = line_difference_2d(line0, line1, EPSILON);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Line2D::new((0.0, 0.0), (1.0, 1.0)));
        assert_eq!(result[1], Line2D::new((2.0, 2.0), (3.0, 3.0)));
    }
}
