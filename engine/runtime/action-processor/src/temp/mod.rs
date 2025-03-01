use reearth_flow_geometry::types::{coordinate::Coordinate2D, line::{Line, Line2D}};

const EPSILON: f64 = 1e-6;

pub fn line_length_2d(line: Line2D<f64>) -> f64 {
    let delta = line.delta();
    (delta.x * delta.x + delta.y * delta.y).sqrt()
}

pub fn point_on_line_2d(line: Line2D<f64>, point: Coordinate2D<f64>) -> bool {
    let line_length = line_length_2d(line);

    let length_1 = line_length_2d(Line::new(line.start, point));
    let length_2 = line_length_2d(Line::new(point, line.end));

    (length_1 + length_2 - line_length).abs() < EPSILON
}

pub fn line_difference_2d(line0: Line2D<f64>, line1: Line2D<f64>) -> Vec<Line2D<f64>> {
    fn lerp(a: Coordinate2D<f64>, b: Coordinate2D<f64>, t: f64) -> Coordinate2D<f64> {
        Coordinate2D::new_(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
    }

    fn t_of(line: &Line2D<f64>, pt: &Coordinate2D<f64>) -> f64 {
        let dx_line = line.end.x - line.start.x;
        let dy_line = line.end.y - line.start.y;
        let dx_pt = pt.x - line.start.x;
        let dy_pt = pt.y - line.start.y;

        let dot = dx_line * dx_pt + dy_line * dy_pt;
        let len_sq = dx_line * dx_line + dy_line * dy_line;

        dot / len_sq
    }

    // 線分が同一直線上にあるか判定 (cross productで判断)
    let cross = (line0.end.x - line0.start.x) * (line1.end.y - line1.start.y)
              - (line0.end.y - line0.start.y) * (line1.end.x - line1.start.x);
    
    if cross.abs() > EPSILON {
        // 交差なし (直線が異なる)
        return vec![line0];
    }

    // 同一直線上と仮定して、それぞれの点の位置を取得 (t座標系)
    let mut t0 = t_of(&line0, &line1.start);
    let mut t1 = t_of(&line0, &line1.end);

    // ソート
    if t0 > t1 { std::mem::swap(&mut t0, &mut t1); }

    let (t_start, t_end) = (0.0, 1.0);

    let overlap_start = t0.clamp(t_start, t_end);
    let overlap_end = t1.clamp(t_start, t_end);

    if overlap_start >= overlap_end {
        // 重複なし
        println!("no overlap");
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


mod tests {
    use super::*;

    #[test]
    fn test_no_overlap() {
        // 重複なし
        let line0 = Line2D::new((0.0, 0.0), (1.0, 1.0));
        let line1 = Line2D::new((2.0, 2.0), (3.0, 3.0));

        let result = line_difference_2d(line0, line1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], line0);
    }

    #[test]
    fn test_full_overlap() {
        // 完全に内包
        let line0 = Line2D::new((0.0, 0.0), (2.0, 2.0));
        let line1 = Line2D::new((0.0, 0.0), (2.0, 2.0));

        let result = line_difference_2d(line0, line1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_partial_overlap_start() {
        // 前半部分が重複
        let line0 = Line2D::new((0.0, 0.0), (2.0, 2.0));
        let line1 = Line2D::new((0.0, 0.0), (1.0, 1.0));

        let result = line_difference_2d(line0, line1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Line2D::new((1.0, 1.0), (2.0, 2.0)));
    }

    #[test]
    fn test_partial_overlap_end() {
        // 後半部分が重複
        let line0 = Line2D::new((0.0, 0.0), (2.0, 2.0));
        let line1 = Line2D::new((1.0, 1.0), (2.0, 2.0));

        let result = line_difference_2d(line0, line1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Line2D::new((0.0, 0.0), (1.0, 1.0)));
    }

    #[test]
    fn test_partial_overlap_middle() {
        // 中央部分が重複
        let line0 = Line2D::new((0.0, 0.0), (3.0, 3.0));
        let line1 = Line2D::new((1.0, 1.0), (2.0, 2.0));

        let result = line_difference_2d(line0, line1);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Line2D::new((0.0, 0.0), (1.0, 1.0)));
        assert_eq!(result[1], Line2D::new((2.0, 2.0), (3.0, 3.0)));
    }

    #[test]
    fn test_overlap_outside() {
        // line1 が line0 より大きい（line0完全に内包される）
        let line0 = Line2D::new((1.0, 1.0), (2.0, 2.0));
        let line1 = Line2D::new((0.0, 0.0), (3.0, 3.0));

        let result = line_difference_2d(line0, line1);
        println!("{:?}", result);
        assert!(result.is_empty());
    }

    #[test]
    fn test_reverse_direction() {
        // 向きが逆でも機能することを確認
        let line0 = Line2D::new((0.0, 0.0), (3.0, 3.0));
        let line1 = Line2D::new((2.0, 2.0), (1.0, 1.0));

        let result = line_difference_2d(line0, line1);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Line2D::new((0.0, 0.0), (1.0, 1.0)));
        assert_eq!(result[1], Line2D::new((2.0, 2.0), (3.0, 3.0)));
    }

    #[test]
    fn test_line_length_2d() {
        let line = Line2D::new(Coordinate2D::new_(0.0, 0.0), Coordinate2D::new_(3.0, 4.0));
        assert_eq!(line_length_2d(line), 5.0);
    }
}