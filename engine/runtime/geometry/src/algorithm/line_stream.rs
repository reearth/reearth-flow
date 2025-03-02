use crate::{
    algorithm::GeoFloat,
    types::{line::Line2D, line_string::LineString2D},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStreamType<T> {
    Line(T),
    Split,
}

/// Joins line segments into continuous line strings.
pub fn join_line_stream_2d<T: GeoFloat>(
    line_stream: Vec<LineStreamType<Line2D<T>>>,
    torelance: T,
) -> Vec<LineString2D<T>> {
    if line_stream.is_empty() {
        return vec![];
    }

    let mut end_point = None;

    let mut line_strings = vec![];
    let mut coords_buffer = vec![];
    for line in line_stream {
        match line {
            LineStreamType::Line(line) => {
                if let Some(end_point) = end_point {
                    if Line2D::new(end_point, line.start).length() < torelance {
                        coords_buffer.push(line.end);
                    } else {
                        line_strings.push(LineString2D::new(coords_buffer.clone()));
                        coords_buffer.clear();
                        coords_buffer.push(line.start);
                        coords_buffer.push(line.end);
                    }
                } else {
                    coords_buffer.push(line.start);
                    coords_buffer.push(line.end);
                }
                end_point = Some(line.end);
            }

            LineStreamType::Split => {
                if let Some(end_point) = end_point {
                    line_strings.push(LineString2D::new(coords_buffer.clone()));
                    coords_buffer.clear();
                    coords_buffer.push(end_point);
                } else {
                    continue;
                };
            }
        }
    }
    line_strings
}
