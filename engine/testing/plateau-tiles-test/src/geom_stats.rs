use reearth_flow_geometry::types::coordinate::Coordinate;

pub fn compute_bbox(
    triangles: &[[usize; 3]],
    positions: &[Coordinate],
) -> Result<(Coordinate, Coordinate), String> {
    if triangles.is_empty() {
        return Err("Cannot compute bbox: no triangles".to_string());
    }
    let mut min = Coordinate::from((f64::INFINITY, f64::INFINITY, f64::INFINITY));
    let mut max = Coordinate::from((f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY));
    for triangle in triangles {
        for &idx in triangle {
            let pos = positions
                .get(idx)
                .ok_or_else(|| format!("Invalid vertex index {}", idx))?;
            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            min.z = min.z.min(pos.z);
            max.x = max.x.max(pos.x);
            max.y = max.y.max(pos.y);
            max.z = max.z.max(pos.z);
        }
    }
    Ok((min, max))
}

pub fn compute_centroid(
    triangles: &[[usize; 3]],
    positions: &[Coordinate],
) -> Result<Coordinate, String> {
    if triangles.is_empty() {
        return Err("Cannot compute centroid: no triangles".to_string());
    }
    let mut weighted_sum = Coordinate::from((0.0, 0.0, 0.0));
    let mut total_area = 0.0;
    for triangle in triangles {
        let p0 = positions[triangle[0]];
        let p1 = positions[triangle[1]];
        let p2 = positions[triangle[2]];
        let tri_centroid = Coordinate::from((
            (p0.x + p1.x + p2.x) / 3.0,
            (p0.y + p1.y + p2.y) / 3.0,
            (p0.z + p1.z + p2.z) / 3.0,
        ));
        let area = (p1 - p0).cross(&(p2 - p0)).norm() / 2.0;
        weighted_sum.x += tri_centroid.x * area;
        weighted_sum.y += tri_centroid.y * area;
        weighted_sum.z += tri_centroid.z * area;
        total_area += area;
    }
    if total_area == 0.0 {
        return Err("Cannot compute centroid: total area is zero".to_string());
    }
    Ok(Coordinate::from((
        weighted_sum.x / total_area,
        weighted_sum.y / total_area,
        weighted_sum.z / total_area,
    )))
}

pub fn compute_area_weighted_winding(
    triangles: &[[usize; 3]],
    positions: &[Coordinate],
) -> Result<[f64; 3], String> {
    if triangles.is_empty() {
        return Err("Cannot compute average winding: no triangles".to_string());
    }
    let mut weighted = [0.0f64; 3];
    let mut total_area = 0.0;
    for triangle in triangles {
        let p0 = positions[triangle[0]];
        let p1 = positions[triangle[1]];
        let p2 = positions[triangle[2]];
        let cross = (p1 - p0).cross(&(p2 - p0));
        weighted[0] += cross.x / 2.0;
        weighted[1] += cross.y / 2.0;
        weighted[2] += cross.z / 2.0;
        total_area += cross.norm() / 2.0;
    }
    if total_area == 0.0 {
        return Err("Cannot compute average winding: total area is zero".to_string());
    }
    Ok([
        weighted[0] / total_area,
        weighted[1] / total_area,
        weighted[2] / total_area,
    ])
}

pub fn compute_total_area(triangles: &[[usize; 3]], positions: &[Coordinate]) -> f64 {
    triangles
        .iter()
        .map(|tri| {
            let p0 = positions[tri[0]];
            let p1 = positions[tri[1]];
            let p2 = positions[tri[2]];
            (p1 - p0).cross(&(p2 - p0)).norm() / 2.0
        })
        .sum()
}
