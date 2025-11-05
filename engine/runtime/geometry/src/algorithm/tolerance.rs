use crate::types::{coordinate::Coordinate, coordnum::CoordFloat};

/// Glue vertices that are closer than the given tolerance.
/// No points will be moved more than the tolerance distance.
/// Each pair of vertices will be at least the tolerance distance apart after this operation.
pub fn glue_vertices_closer_than<T: CoordFloat + From<Z>, Z: CoordFloat>(
    tolerance: T,
    mut vertices: Vec<&mut Coordinate<T, Z>>,
) {
    let mut updated = vec![false; vertices.len()];
    for i in 0..vertices.len() {
        if updated[i] {
            // `vi` is already glued. If some other vertex is glued to the old value of `vi` but not to the vertex which `vi` is glued to,
            // then it will be at least `tolerance` away from the new value of `vi`, so we can skip this.
            continue;
        }
        let vi = *vertices[i];
        for j in (i + 1)..vertices.len() {
            if updated[j] {
                // `vj` is already glued to some vertex `vi`. If 'vj' can be glued to some other vertex, say `vk`, then
                // `vi==vj` implies that `vi` and `vk` must be at most `tolerance` away, which in turn implies that `vi`
                // must have been glued to `vk` already in an earlier iteration, a contradiction. So we can safely skip this.
                continue;
            }
            let vj = *vertices[j];
            if (vi - vj).norm() < tolerance {
                // Glue vj to vi
                *vertices[j] = vi;
                updated[j] = true;
            }
        }
    }
}
