use image::RgbaImage;

use super::Rect;

pub(crate) fn fill_frame_extrusion(atlas: &mut RgbaImage, frame: Rect, extrusion: u32) {
    if extrusion == 0 || frame.w == 0 || frame.h == 0 {
        return;
    }
    let left = frame.x;
    let top = frame.y;
    let right = frame.right() - 1;
    let bottom = frame.bottom() - 1;
    let fill_left = left.saturating_sub(extrusion);
    let fill_top = top.saturating_sub(extrusion);
    let fill_right = right.saturating_add(extrusion).min(atlas.width() - 1);
    let fill_bottom = bottom.saturating_add(extrusion).min(atlas.height() - 1);

    for y in top..=bottom {
        let a = *atlas.get_pixel(left, y);
        let b = *atlas.get_pixel(right, y);
        for x in fill_left..left {
            atlas.put_pixel(x, y, a);
        }
        for x in right + 1..=fill_right {
            atlas.put_pixel(x, y, b);
        }
    }
    for x in fill_left..=fill_right {
        let a = *atlas.get_pixel(x, top);
        let b = *atlas.get_pixel(x, bottom);
        for y in fill_top..top {
            atlas.put_pixel(x, y, a);
        }
        for y in bottom + 1..=fill_bottom {
            atlas.put_pixel(x, y, b);
        }
    }
}

#[cfg(test)]
mod tests {
    use image::Rgba;

    use super::*;

    #[test]
    fn test_fill_frame_extrusion_replicates_edge_pixels() {
        let mut atlas = RgbaImage::new(4, 4);
        atlas.put_pixel(1, 1, Rgba([10, 11, 12, 255]));
        atlas.put_pixel(2, 1, Rgba([20, 21, 22, 255]));
        atlas.put_pixel(1, 2, Rgba([30, 31, 32, 255]));
        atlas.put_pixel(2, 2, Rgba([40, 41, 42, 255]));
        fill_frame_extrusion(
            &mut atlas,
            Rect {
                x: 1,
                y: 1,
                w: 2,
                h: 2,
            },
            1,
        );
        assert_eq!(*atlas.get_pixel(0, 0), Rgba([10, 11, 12, 255]));
        assert_eq!(*atlas.get_pixel(3, 0), Rgba([20, 21, 22, 255]));
        assert_eq!(*atlas.get_pixel(0, 3), Rgba([30, 31, 32, 255]));
        assert_eq!(*atlas.get_pixel(3, 3), Rgba([40, 41, 42, 255]));
    }
}
