use super::Rect;

#[derive(Debug)]
struct Skyline {
    x: u32,
    y: u32,
    w: u32,
}

#[derive(Debug)]
pub struct SkylinePacker {
    max_w: u32,
    max_h: u32,
    extrusion: u32,
    used_w: u32,
    used_h: u32,
    skylines: Vec<Skyline>,
}

impl SkylinePacker {
    pub fn new(max_w: u32, max_h: u32, extrusion: u32) -> Self {
        Self {
            max_w,
            max_h,
            extrusion,
            used_w: 0,
            used_h: 0,
            skylines: vec![Skyline {
                x: 0,
                y: 0,
                w: max_w,
            }],
        }
    }

    pub fn width(&self) -> u32 {
        self.used_w.max(1)
    }

    pub fn height(&self) -> u32 {
        self.used_h.max(1)
    }

    pub fn pack(&mut self, w: u32, h: u32) -> Option<Rect> {
        let (i, region) = self.find(w, h)?;
        // Region sits flush; reserve a 2*extrusion gap only right and below, so
        // neighbours' extrusion rings never overlap and page edges aren't padded.
        let gap = self.extrusion * 2;
        let footprint = Rect {
            x: region.x,
            y: region.y,
            w: (region.w + gap).min(self.max_w - region.x),
            h: (region.h + gap).min(self.max_h - region.y),
        };
        self.split(i, footprint);
        self.merge();
        self.used_w = self.used_w.max(region.x + region.w);
        self.used_h = self.used_h.max(region.y + region.h);
        Some(region)
    }

    fn find(&self, w: u32, h: u32) -> Option<(usize, Rect)> {
        let mut best = None;
        let mut bottom = u32::MAX;
        let mut width = u32::MAX;
        for i in 0..self.skylines.len() {
            let Some(rect) = self.can_put(i, w, h) else {
                continue;
            };
            if rect.bottom() < bottom || (rect.bottom() == bottom && self.skylines[i].w < width) {
                bottom = rect.bottom();
                width = self.skylines[i].w;
                best = Some((i, rect));
            }
        }
        best
    }

    fn can_put(&self, i: usize, w: u32, h: u32) -> Option<Rect> {
        let mut rect = Rect {
            x: self.skylines[i].x,
            y: 0,
            w,
            h,
        };
        let mut left = w;
        for skyline in &self.skylines[i..] {
            rect.y = rect.y.max(skyline.y);
            if rect.x + rect.w > self.max_w || rect.y + rect.h > self.max_h {
                return None;
            }
            if skyline.w >= left {
                return Some(rect);
            }
            left -= skyline.w;
        }
        tracing::error!("skyline invariant violated: skylines do not cover full width");
        None
    }

    fn split(&mut self, index: usize, rect: Rect) {
        self.skylines.insert(
            index,
            Skyline {
                x: rect.x,
                y: rect.y + rect.h,
                w: rect.w,
            },
        );
        let i = index + 1;
        while i < self.skylines.len() {
            let prev_right = self.skylines[i - 1].x + self.skylines[i - 1].w;
            if self.skylines[i].x >= prev_right {
                break;
            }
            let shrink = prev_right - self.skylines[i].x;
            if self.skylines[i].w <= shrink {
                self.skylines.remove(i);
            } else {
                self.skylines[i].x += shrink;
                self.skylines[i].w -= shrink;
                break;
            }
        }
    }

    fn merge(&mut self) {
        let mut i = 1;
        while i < self.skylines.len() {
            if self.skylines[i - 1].y == self.skylines[i].y {
                self.skylines[i - 1].w += self.skylines[i].w;
                self.skylines.remove(i);
            } else {
                i += 1;
            }
        }
    }
}
