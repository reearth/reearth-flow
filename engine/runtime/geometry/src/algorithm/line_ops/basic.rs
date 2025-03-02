use crate::{
    algorithm::GeoNum,
    types::{
        coordinate::Coordinate2D,
        line::{Line, Line2D},
    },
};

pub fn line_length_2d<T: GeoNum>(line: Line2D<T>) -> T {
    let delta = line.delta();
    (delta.x * delta.x + delta.y * delta.y).sqrt()
}

pub fn point_on_line_2d<T: GeoNum>(line: Line2D<T>, point: Coordinate2D<T>, torelance: T) -> bool {
    let line_length = line_length_2d(line);

    let length_1 = line_length_2d(Line::new(line.start, point));
    let length_2 = line_length_2d(Line::new(point, line.end));

    (length_1 + length_2 - line_length).abs() < torelance
}
