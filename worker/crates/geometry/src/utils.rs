use crate::{
    coord,
    types::{
        coordinate::Coordinate, coordnum::CoordNum, line::Line, line_string::LineString,
        no_value::NoValue, rectangle::Rectangle,
    },
};

pub fn line_string_bounding_rectangle<T>(
    line_string: &LineString<T, NoValue>,
) -> Option<Rectangle<T, NoValue>>
where
    T: CoordNum,
{
    get_bounding_rectangle(line_string.coords().cloned())
}

pub fn line_bounding_rectangle<T>(line: Line<T>) -> Rectangle<T>
where
    T: CoordNum,
{
    Rectangle::new(line.start, line.end)
}

pub fn get_bounding_rectangle<I, T>(collection: I) -> Option<Rectangle<T, NoValue>>
where
    T: CoordNum,
    I: IntoIterator<Item = Coordinate<T, NoValue>>,
{
    let mut iter = collection.into_iter();
    if let Some(pnt) = iter.next() {
        let mut xrange = (pnt.x, pnt.x);
        let mut yrange = (pnt.y, pnt.y);
        for pnt in iter {
            let (px, py) = pnt.x_y();
            xrange = get_min_max(px, xrange.0, xrange.1);
            yrange = get_min_max(py, yrange.0, yrange.1);
        }

        return Some(Rectangle::new(
            coord! {
                x: xrange.0,
                y: yrange.0,
            },
            coord! {
                x: xrange.1,
                y: yrange.1,
            },
        ));
    }
    None
}

fn get_min_max<T: PartialOrd>(p: T, min: T, max: T) -> (T, T) {
    if p > max {
        (min, p)
    } else if p < min {
        (p, max)
    } else {
        (min, max)
    }
}
