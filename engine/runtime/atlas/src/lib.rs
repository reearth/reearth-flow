mod blit;
mod damage;
mod error;
mod multipage;
mod skyline;

use std::path::PathBuf;

pub use error::{AtlasError, Result};
pub use multipage::{build_atlas_multipage, MultiPageAtlas, PageWrap, PolygonPlacement, TextureCache};

pub type PolygonUVs = Vec<[f64; 2]>;
pub type TextureUVs = Vec<PolygonUVs>;

/// Axis-aligned rectangle in atlas pixel space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn right(self) -> u32 {
        self.x + self.w
    }

    pub fn bottom(self) -> u32 {
        self.y + self.h
    }

    pub fn union(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self {
            x,
            y,
            w: self.right().max(other.right()) - x,
            h: self.bottom().max(other.bottom()) - y,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextureInput {
    pub path: PathBuf,
    pub uvs: TextureUVs,
    /// Fraction of native resolution to keep when packing (`(0, 1]`; `1.0` =
    /// full resolution).
    pub scale: f64,
}

struct RemapContext {
    texture_size: (u32, u32),
    damage: Rect,
    frame: Rect,
    atlas_size: (f64, f64),
}

fn remap_uv(u: f64, v: f64, ctx: &RemapContext) -> [f64; 2] {
    // Map the source region's pixel span onto its atlas frame independently per
    // axis. The multipage packer rounds frame width and height separately, so a
    // single (horizontal) scale factor drifts the vertical axis; per-axis ratios
    // map the region edges onto the frame edges exactly regardless of rounding.
    let sx = ctx.frame.w as f64 / ctx.damage.w as f64;
    let sy = ctx.frame.h as f64 / ctx.damage.h as f64;
    let px = u * ctx.texture_size.0 as f64 - ctx.damage.x as f64;
    let py = (1.0 - v) * ctx.texture_size.1 as f64 - ctx.damage.y as f64;
    let out_u = (ctx.frame.x as f64 + px * sx) / ctx.atlas_size.0;
    let row = (ctx.frame.y as f64 + py * sy) / ctx.atlas_size.1;
    [out_u, 1.0 - row]
}

pub(crate) fn remap_polygon_uvs(
    poly_uvs: &PolygonUVs,
    texture_size: (u32, u32),
    damage: Rect,
    frame: Rect,
    atlas_size: (f64, f64),
) -> PolygonUVs {
    let ctx = RemapContext {
        texture_size,
        damage,
        frame,
        atlas_size,
    };
    poly_uvs
        .iter()
        .map(|&[u, v]| remap_uv(u, v, &ctx))
        .collect()
}
