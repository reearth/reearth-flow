use std::collections::HashMap;
use std::path::PathBuf;

use super::TextureMaterial;

/// Axis-aligned rectangle in source texture pixel space (origin top-left).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl DamageRect {
    pub fn right(self) -> u32 {
        self.x + self.w
    }

    pub fn bottom(self) -> u32 {
        self.y + self.h
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
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

    /// Expand to 2^k pixel alignment, clamped to texture bounds.
    pub fn align(self, k: u32, tex_w: u32, tex_h: u32) -> Self {
        if k == 0 {
            return self;
        }
        let align = 1u32 << k;
        let x = (self.x / align) * align;
        let y = (self.y / align) * align;
        let right = (self.right().div_ceil(align) * align).min(tex_w);
        let bottom = (self.bottom().div_ceil(align) * align).min(tex_h);
        Self {
            x,
            y,
            w: right - x,
            h: bottom - y,
        }
    }
}

pub struct TextureDamage {
    pub width: u32,
    pub height: u32,
    /// Disjoint merged damage rects, sorted by area descending.
    pub rects: Vec<DamageRect>,
    /// For each polygon in `TextureMaterial::uvs`, the merged rect index it belongs to.
    pub polygon_regions: Vec<usize>,
}

struct DamageRegion {
    rect: DamageRect,
    polygons: Vec<usize>,
}

/// Merge a list of potentially overlapping polygon regions into disjoint rects.
fn merge_regions(mut regions: Vec<DamageRegion>) -> Vec<DamageRegion> {
    loop {
        let mut merged = false;
        let mut result: Vec<DamageRegion> = Vec::with_capacity(regions.len());
        'outer: for region in regions.drain(..) {
            for existing in &mut result {
                if existing.rect.overlaps(region.rect) {
                    existing.rect = existing.rect.union(region.rect);
                    existing.polygons.extend(region.polygons);
                    merged = true;
                    continue 'outer;
                }
            }
            result.push(region);
        }
        regions = result;
        if !merged {
            break;
        }
    }
    regions
}

/// Collect per-texture damage rectangles from polygon UV coverages.
pub fn collect_damage(
    materials: &[TextureMaterial],
    k: u32,
) -> crate::Result<Vec<(PathBuf, TextureDamage)>> {
    let mut candidates: HashMap<PathBuf, Vec<DamageRegion>> = HashMap::new();
    let mut dims: HashMap<PathBuf, (u32, u32)> = HashMap::new();

    for mat in materials {
        for (polygon_idx, poly_uvs) in mat.uvs.iter().enumerate() {
            let (tw, th) = match dims.get(&mat.path) {
                Some(&d) => d,
                None => {
                    let (w, h) = image::image_dimensions(&mat.path).map_err(|e| {
                        crate::AtlasError::builder(format!(
                            "Failed to read image dimensions for '{}': {e}",
                            mat.path.display()
                        ))
                    })?;
                    dims.insert(mat.path.clone(), (w, h));
                    (w, h)
                }
            };

            let (min_u, max_u, min_v, max_v) = poly_uvs.iter().fold(
                (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
                |(mn_u, mx_u, mn_v, mx_v), [u, v]| {
                    (mn_u.min(*u), mx_u.max(*u), mn_v.min(*v), mx_v.max(*v))
                },
            );

            let x = ((min_u * tw as f64).floor().max(0.0) as u32).min(tw);
            let y = (((1.0 - max_v) * th as f64).floor().max(0.0) as u32).min(th);
            let right = ((max_u * tw as f64).ceil() as u32).min(tw);
            let bottom = (((1.0 - min_v) * th as f64).ceil() as u32).min(th);

            if right <= x || bottom <= y {
                continue;
            }

            let rect = DamageRect {
                x,
                y,
                w: right - x,
                h: bottom - y,
            }
            .align(k, tw, th);
            candidates
                .entry(mat.path.clone())
                .or_default()
                .push(DamageRegion {
                    rect,
                    polygons: vec![polygon_idx],
                });
        }
    }

    let mut result: Vec<(PathBuf, TextureDamage)> = candidates
        .into_iter()
        .filter_map(|(path, regions)| {
            let &(width, height) = dims.get(&path)?;
            let mut merged = merge_regions(regions);
            merged.sort_by(|a, b| {
                (b.rect.w as u64 * b.rect.h as u64).cmp(&(a.rect.w as u64 * a.rect.h as u64))
            });
            let mut polygon_regions = vec![
                0;
                merged
                    .iter()
                    .flat_map(|r| r.polygons.iter())
                    .max()
                    .map_or(0, |i| i + 1)
            ];
            let rects = merged
                .into_iter()
                .enumerate()
                .map(|(region_idx, region)| {
                    for polygon_idx in region.polygons {
                        polygon_regions[polygon_idx] = region_idx;
                    }
                    region.rect
                })
                .collect();
            Some((
                path,
                TextureDamage {
                    width,
                    height,
                    rects,
                    polygon_regions,
                },
            ))
        })
        .collect();

    result.sort_by(|a, b| {
        let area = |td: &TextureDamage| {
            td.rects
                .iter()
                .map(|r| r.w as u64 * r.h as u64)
                .sum::<u64>()
        };
        area(&b.1).cmp(&area(&a.1))
    });

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    use super::super::TextureMaterial;

    fn create_texture(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
        use image::{ImageBuffer, Rgb};
        let img = ImageBuffer::<Rgb<u8>, _>::new(w, h);
        let path = dir.join(name);
        img.save(&path).unwrap();
        path
    }

    fn make_material(path: PathBuf, uvs: &[(f64, f64)]) -> TextureMaterial {
        TextureMaterial {
            path,
            uvs: vec![uvs.iter().map(|&(u, v)| [u, v]).collect()],
        }
    }

    #[test]
    fn test_large_texture_included() {
        let tmp = TempDir::new().unwrap();
        let path = create_texture(tmp.path(), "large.png", 16384, 1);
        let mat = make_material(path, &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let result = collect_damage(&[mat], 0).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.width, 16384);
        assert_eq!(result[0].1.polygon_regions, vec![0]);
    }

    #[test]
    fn test_disjoint_uvs_produce_two_rects() {
        let tmp = TempDir::new().unwrap();
        let path = create_texture(tmp.path(), "t.png", 100, 100);
        let mat = TextureMaterial {
            path,
            uvs: vec![
                vec![[0.0, 0.5], [0.3, 0.5], [0.3, 1.0], [0.0, 1.0]],
                vec![[0.7, 0.0], [1.0, 0.0], [1.0, 0.5], [0.7, 0.5]],
            ],
        };
        let result = collect_damage(&[mat], 0).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].1.rects.len(),
            2,
            "non-overlapping regions must not merge"
        );
        assert_eq!(result[0].1.polygon_regions, vec![0, 1]);
    }

    #[test]
    fn test_overlapping_uvs_merge() {
        let tmp = TempDir::new().unwrap();
        let path = create_texture(tmp.path(), "t.png", 100, 100);
        let mat = TextureMaterial {
            path,
            uvs: vec![
                vec![[0.0, 0.0], [0.6, 0.0], [0.6, 1.0], [0.0, 1.0]],
                vec![[0.4, 0.0], [1.0, 0.0], [1.0, 1.0], [0.4, 1.0]],
            ],
        };
        let result = collect_damage(&[mat], 0).unwrap();
        assert_eq!(result[0].1.rects.len(), 1, "overlapping regions must merge");
        assert_eq!(result[0].1.polygon_regions, vec![0, 0]);
    }

    #[test]
    fn test_align_k() {
        let r = DamageRect {
            x: 3,
            y: 5,
            w: 10,
            h: 9,
        };
        let aligned = r.align(2, 64, 64);
        assert_eq!(
            aligned,
            DamageRect {
                x: 0,
                y: 4,
                w: 16,
                h: 12
            }
        );
    }
}
