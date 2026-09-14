use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::PathBuf;

use rstar::{RTree, RTreeObject, AABB};

use super::{Rect, TextureInput};

#[derive(Debug, Clone)]
pub struct TextureDamage {
    /// Full source image dimensions (pixels), used for UV→pixel conversion.
    pub src_width: u32,
    pub src_height: u32,
    /// Disjoint merged damage rects, sorted by area descending.
    pub rects: Vec<Rect>,
    /// For each polygon in `TextureInput::uvs`, the merged rect index it belongs to.
    pub polygon_regions: Vec<usize>,
}

struct DamageRegion {
    rect: Rect,
    polygons: Vec<usize>,
}

struct REntry {
    idx: usize,
    rect: Rect,
}

impl RTreeObject for REntry {
    type Envelope = AABB<[f64; 2]>;
    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.rect.x as f64, self.rect.y as f64],
            [self.rect.right() as f64, self.rect.bottom() as f64],
        )
    }
}

/// Merge a list of potentially overlapping polygon regions into disjoint rects.
fn merge_regions(regions: Vec<DamageRegion>) -> Vec<DamageRegion> {
    let tree = RTree::bulk_load(
        regions
            .iter()
            .enumerate()
            .map(|(i, r)| REntry {
                idx: i,
                rect: r.rect,
            })
            .collect(),
    );
    let mut used = vec![false; regions.len()];
    let mut result = Vec::new();
    for start in 0..regions.len() {
        if used[start] {
            continue;
        }
        used[start] = true;
        let mut merged = regions[start].rect;
        let mut polys = regions[start].polygons.clone();
        loop {
            let env = AABB::from_corners(
                [merged.x as f64, merged.y as f64],
                [merged.right() as f64, merged.bottom() as f64],
            );
            let found: Vec<_> = tree
                .locate_in_envelope_intersecting(&env)
                .filter(|e| !used[e.idx])
                .map(|e| e.idx)
                .collect();
            if found.is_empty() {
                break;
            }
            for idx in found {
                used[idx] = true;
                merged = merged.union(regions[idx].rect);
                polys.extend_from_slice(&regions[idx].polygons);
            }
        }
        result.push(DamageRegion {
            rect: merged,
            polygons: polys,
        });
    }
    result
}

/// Collect per-texture damage rectangles from polygon UV coverages.
pub fn collect_damage(materials: &[TextureInput]) -> crate::Result<Vec<(PathBuf, TextureDamage)>> {
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

            if min_u < 0.0 || max_u > 1.0 || min_v < 0.0 || max_v > 1.0 {
                tracing::error!(
                    "reearth-flow-atlas: polygon {} of '{}' has UV coordinates outside \
                     [0,1] (u=[{min_u:.4},{max_u:.4}], v=[{min_v:.4},{max_v:.4}]); clamping",
                    polygon_idx,
                    mat.path.display()
                );
            }
            let min_u = min_u.clamp(0.0, 1.0);
            let max_u = max_u.clamp(0.0, 1.0);
            let min_v = min_v.clamp(0.0, 1.0);
            let max_v = max_v.clamp(0.0, 1.0);

            let x = ((min_u * tw as f64).floor() as u32).min(tw);
            let y = (((1.0 - max_v) * th as f64).floor() as u32).min(th);
            let right = ((max_u * tw as f64).ceil() as u32).min(tw);
            let bottom = (((1.0 - min_v) * th as f64).ceil() as u32).min(th);

            // Every polygon must be represented — guarantee a minimum 1×1 damage rect
            // so that polygon_regions is dense and no index is ever left unmapped.
            let x = x.min(tw.saturating_sub(1));
            let y = y.min(th.saturating_sub(1));
            let right = right.max(x + 1);
            let bottom = bottom.max(y + 1);

            let rect = Rect {
                x,
                y,
                w: right - x,
                h: bottom - y,
            };
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
                    src_width: width,
                    src_height: height,
                    rects,
                    polygon_regions,
                },
            ))
        })
        .collect();

    result.sort_by_cached_key(|(_, td)| {
        Reverse(
            td.rects
                .iter()
                .map(|r| r.w as u64 * r.h as u64)
                .sum::<u64>(),
        )
    });

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    use super::super::TextureInput;

    fn create_texture(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
        use image::{ImageBuffer, Rgb};
        let img = ImageBuffer::<Rgb<u8>, _>::new(w, h);
        let path = dir.join(name);
        img.save(&path).unwrap();
        path
    }

    fn make_material(path: PathBuf, uvs: &[(f64, f64)]) -> TextureInput {
        TextureInput {
            path,
            uvs: vec![uvs.iter().map(|&(u, v)| [u, v]).collect()],
        }
    }

    #[test]
    fn test_large_texture_included() {
        let tmp = TempDir::new().unwrap();
        let path = create_texture(tmp.path(), "large.png", 16384, 1);
        let mat = make_material(path, &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let result = collect_damage(&[mat]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.src_width, 16384);
        assert_eq!(result[0].1.polygon_regions, vec![0]);
    }

    #[test]
    fn test_disjoint_uvs_produce_two_rects() {
        let tmp = TempDir::new().unwrap();
        let path = create_texture(tmp.path(), "t.png", 100, 100);
        let mat = TextureInput {
            path,
            uvs: vec![
                vec![[0.0, 0.5], [0.3, 0.5], [0.3, 1.0], [0.0, 1.0]],
                vec![[0.7, 0.0], [1.0, 0.0], [1.0, 0.5], [0.7, 0.5]],
            ],
        };
        let result = collect_damage(&[mat]).unwrap();
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
        let mat = TextureInput {
            path,
            uvs: vec![
                vec![[0.0, 0.0], [0.6, 0.0], [0.6, 1.0], [0.0, 1.0]],
                vec![[0.4, 0.0], [1.0, 0.0], [1.0, 1.0], [0.4, 1.0]],
            ],
        };
        let result = collect_damage(&[mat]).unwrap();
        assert_eq!(result[0].1.rects.len(), 1, "overlapping regions must merge");
        assert_eq!(result[0].1.polygon_regions, vec![0, 0]);
    }
}
