use super::damage::Rect;

struct Skyline {
    x: u32,
    y: u32,
    w: u32,
}

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
        let w = w + self.extrusion * 2;
        let h = h + self.extrusion * 2;
        let (i, rect) = self.find(w, h)?;
        self.split(i, rect);
        self.merge();
        self.used_w = self.used_w.max(rect.x + rect.w);
        self.used_h = self.used_h.max(rect.y + rect.h);
        Some(Rect {
            x: rect.x + self.extrusion,
            y: rect.y + self.extrusion,
            w: rect.w - self.extrusion * 2,
            h: rect.h - self.extrusion * 2,
        })
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

    fn can_put(&self, mut i: usize, w: u32, h: u32) -> Option<Rect> {
        let mut rect = Rect {
            x: self.skylines[i].x,
            y: 0,
            w,
            h,
        };
        let mut left = w;
        loop {
            rect.y = rect.y.max(self.skylines[i].y);
            if rect.x + rect.w > self.max_w || rect.y + rect.h > self.max_h {
                return None;
            }
            if self.skylines[i].w >= left {
                return Some(rect);
            }
            left -= self.skylines[i].w;
            i += 1;
        }
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
