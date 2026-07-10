//! `.subtree` binary encoding (3D Tiles 1.1 implicit tiling): the
//! `tileAvailability` / `contentAvailability` / `childSubtreeAvailability`
//! bitstreams telling a client which cells exist, without listing them in
//! JSON. Each file covers a fixed `SUBTREE_LEVELS` window; a boundary tile
//! with occupied descendants past that window chains to another `.subtree`
//! file rooted at itself, so no single file scales with dataset depth.

use std::collections::BTreeSet;

use serde_json::{json, Value};

use super::quadtree::Cell;

const MAGIC: &[u8; 4] = b"subt";
const VERSION: u32 = 1;

/// Levels covered by one `.subtree` file. Fixed and small on purpose: memory
/// per file is O(4^SUBTREE_LEVELS), independent of how deep the dataset goes.
pub(super) const SUBTREE_LEVELS: u32 = 4;

/// Every `.subtree` file needed to describe `occupied`, as (root cell, bytes)
/// pairs — one per chain hop.
pub(super) fn build_all(occupied: &BTreeSet<Cell>) -> Vec<(Cell, Vec<u8>)> {
    let mut out = Vec::new();
    build_one(Cell::root(), occupied, &mut out);
    out
}

fn build_one(root: Cell, occupied: &BTreeSet<Cell>, out: &mut Vec<(Cell, Vec<u8>)>) {
    let boundary_level = root.level + SUBTREE_LEVELS - 1;
    let total_tiles = level_offset(SUBTREE_LEVELS) as usize;
    let mut tile_availability = BitSet::new(total_tiles);
    let mut content_availability = BitSet::new(total_tiles);
    let mut chained: BTreeSet<Cell> = BTreeSet::new();

    for cell in occupied
        .iter()
        .copied()
        .filter(|c| c.ancestor_at(root.level) == Some(root))
    {
        // Ancestor within this file's own window (root..=boundary_level) to mark
        // tileAvailability for — the cell itself if it's inside the window, or its
        // boundary-level ancestor if a chained subtree will cover it more deeply.
        let mark_from = cell
            .ancestor_at(cell.level.min(boundary_level))
            .expect("cell is a descendant of root");
        if cell.level > boundary_level {
            // A chained subtree is rooted one level past the boundary, not at the
            // boundary tile itself.
            chained.insert(
                cell.ancestor_at(boundary_level + 1)
                    .expect("cell is deeper than boundary_level + 1"),
            );
        } else {
            content_availability.set(rel_bit_index(root, cell));
        }
        let mut ancestor = Some(mark_from);
        while let Some(c) = ancestor {
            tile_availability.set(rel_bit_index(root, c));
            ancestor = (c != root).then(|| c.parent().expect("non-root cell has a parent"));
        }
    }

    let tile_available_count = tile_availability.count_ones();
    let content_available_count = content_availability.count_ones();
    let mut buffers = vec![
        tile_availability.into_bytes(),
        content_availability.into_bytes(),
    ];

    // childSubtreeAvailability is indexed over the level one past the boundary
    // (where a chained subtree's root would sit), not the boundary level itself.
    let child_json = if chained.is_empty() {
        json!({"constant": 0})
    } else {
        let child_level_tile_count = 4u64.pow(SUBTREE_LEVELS) as usize;
        let mut child_availability = BitSet::new(child_level_tile_count);
        for &c in &chained {
            let (rx, ry) = rel_xy(root, c);
            child_availability.set(morton2d(rx, ry));
        }
        let available_count = child_availability.count_ones();
        let bitstream = buffers.len();
        buffers.push(child_availability.into_bytes());
        json!({"bitstream": bitstream, "availableCount": available_count})
    };

    out.push((
        root,
        encode(
            buffers,
            tile_available_count,
            content_available_count,
            child_json,
        ),
    ));
    for child_root in chained {
        build_one(child_root, occupied, out);
    }
}

fn encode(
    buffers: Vec<Vec<u8>>,
    tile_available_count: usize,
    content_available_count: usize,
    child_json: Value,
) -> Vec<u8> {
    let mut buffer_views = Vec::with_capacity(buffers.len());
    let mut offset = 0usize;
    for b in &buffers {
        buffer_views.push(json!({"buffer": 0, "byteOffset": offset, "byteLength": b.len()}));
        offset += b.len();
    }

    let json = json!({
        "buffers": [{"byteLength": offset}],
        "bufferViews": buffer_views,
        "tileAvailability": {"bitstream": 0, "availableCount": tile_available_count},
        "contentAvailability": [{"bitstream": 1, "availableCount": content_available_count}],
        "childSubtreeAvailability": child_json,
    });

    let mut json_bytes = serde_json::to_vec(&json).expect("subtree JSON is always serializable");
    while !json_bytes.len().is_multiple_of(8) {
        json_bytes.push(b' ');
    }

    let mut binary: Vec<u8> = buffers.into_iter().flatten().collect();
    while !binary.len().is_multiple_of(8) {
        binary.push(0);
    }

    let mut out = Vec::with_capacity(24 + json_bytes.len() + binary.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&(json_bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(&(binary.len() as u64).to_le_bytes());
    out.extend_from_slice(&json_bytes);
    out.extend_from_slice(&binary);
    out
}

/// Number of tiles across relative levels `0..level` of a complete quadtree
/// — the bit offset where `level`'s span starts in the availability bitstream.
fn level_offset(level: u32) -> u64 {
    (4u64.pow(level) - 1) / 3
}

/// `cell`'s coordinates relative to `root`'s window, as `(x, y)` within
/// `cell`'s relative level.
fn rel_xy(root: Cell, cell: Cell) -> (u32, u32) {
    let rel_level = cell.level - root.level;
    let mask = if rel_level == 0 {
        0
    } else {
        (1u32 << rel_level) - 1
    };
    (cell.x & mask, cell.y & mask)
}

/// `cell`'s bit index within `root`'s availability bitstream.
fn rel_bit_index(root: Cell, cell: Cell) -> u64 {
    let rel_level = cell.level - root.level;
    let (rx, ry) = rel_xy(root, cell);
    level_offset(rel_level) + morton2d(rx, ry)
}

fn morton2d(x: u32, y: u32) -> u64 {
    spread_bits(x) | (spread_bits(y) << 1)
}

/// Spreads a 32-bit value's bits into the even bit positions of a 64-bit one.
fn spread_bits(v: u32) -> u64 {
    let mut x = v as u64;
    x = (x | (x << 16)) & 0x0000_FFFF_0000_FFFF;
    x = (x | (x << 8)) & 0x00FF_00FF_00FF_00FF;
    x = (x | (x << 4)) & 0x0F0F_0F0F_0F0F_0F0F;
    x = (x | (x << 2)) & 0x3333_3333_3333_3333;
    x = (x | (x << 1)) & 0x5555_5555_5555_5555;
    x
}

struct BitSet {
    bits: Vec<u8>,
}

impl BitSet {
    fn new(len_bits: usize) -> Self {
        Self {
            bits: vec![0u8; len_bits.div_ceil(8)],
        }
    }

    fn set(&mut self, index: u64) {
        let i = index as usize;
        self.bits[i / 8] |= 1 << (i % 8);
    }

    fn count_ones(&self) -> usize {
        self.bits.iter().map(|b| b.count_ones() as usize).sum()
    }

    fn into_bytes(self) -> Vec<u8> {
        self.bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn morton_interleaves_x_in_even_bits_y_in_odd_bits() {
        assert_eq!(morton2d(0, 0), 0);
        assert_eq!(morton2d(1, 0), 1); // x bit 0 -> bit 0
        assert_eq!(morton2d(0, 1), 2); // y bit 0 -> bit 1
        assert_eq!(morton2d(1, 1), 3);
        assert_eq!(morton2d(2, 0), 4); // x bit 1 -> bit 2
        assert_eq!(morton2d(0, 2), 8); // y bit 1 -> bit 3
    }

    #[test]
    fn level_offset_matches_cumulative_tile_count() {
        assert_eq!(level_offset(0), 0);
        assert_eq!(level_offset(1), 1); // 1 tile at level 0
        assert_eq!(level_offset(2), 5); // + 4 tiles at level 1
        assert_eq!(level_offset(3), 21); // + 16 tiles at level 2
    }

    fn header_lengths(bytes: &[u8]) -> (usize, usize) {
        let json_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
        let binary_len = u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as usize;
        (json_len, binary_len)
    }

    #[test]
    fn header_lengths_match_actual_chunk_sizes() {
        let occupied = BTreeSet::from([Cell {
            level: 1,
            x: 0,
            y: 0,
        }]);
        let files = build_all(&occupied);
        assert_eq!(files.len(), 1);
        let bytes = &files[0].1;
        let (json_len, binary_len) = header_lengths(bytes);
        assert_eq!(&bytes[0..4], MAGIC);
        assert_eq!(bytes.len(), 24 + json_len + binary_len);
        assert_eq!(json_len % 8, 0);
        assert_eq!(binary_len % 8, 0);
    }

    #[test]
    fn root_and_placed_cell_are_both_tile_available_only_placed_is_content_available() {
        let occupied = BTreeSet::from([Cell {
            level: 1,
            x: 1,
            y: 0,
        }]);
        let files = build_all(&occupied);
        assert_eq!(files.len(), 1);
        let bytes = &files[0].1;
        let (json_len, _) = header_lengths(bytes);
        let binary = &bytes[24 + json_len..];
        let tile_availability_len = level_offset(SUBTREE_LEVELS).div_ceil(8) as usize;
        let tile_availability = &binary[..tile_availability_len];
        let content_availability = &binary[tile_availability_len..];

        let bit = |bytes: &[u8], i: u64| (bytes[(i / 8) as usize] >> (i % 8)) & 1;
        let root = Cell::root();
        let placed = Cell {
            level: 1,
            x: 1,
            y: 0,
        };
        let sibling = Cell {
            level: 1,
            x: 0,
            y: 0,
        };

        assert_eq!(bit(tile_availability, rel_bit_index(root, root)), 1);
        assert_eq!(bit(tile_availability, rel_bit_index(root, placed)), 1);
        assert_eq!(bit(tile_availability, rel_bit_index(root, sibling)), 0);

        assert_eq!(bit(content_availability, rel_bit_index(root, root)), 0);
        assert_eq!(bit(content_availability, rel_bit_index(root, placed)), 1);
    }

    #[test]
    fn cell_past_one_window_chains_into_a_second_subtree_file() {
        let deep = Cell {
            level: SUBTREE_LEVELS + 2,
            x: 0,
            y: 0,
        };
        let occupied = BTreeSet::from([deep]);
        let files = build_all(&occupied);

        // One file for the root window, one chained file rooted one level past
        // the boundary, on the path down to `deep`.
        assert_eq!(files.len(), 2);
        let chained_root = deep.ancestor_at(SUBTREE_LEVELS).unwrap();
        assert_eq!(files[1].0, chained_root);

        let (json_len, _) = header_lengths(&files[0].1);
        let binary = &files[0].1[24 + json_len..];
        let tile_availability_len = level_offset(SUBTREE_LEVELS).div_ceil(8) as usize;
        let content_availability =
            &binary[tile_availability_len..tile_availability_len + tile_availability_len];
        // The root window never sees `deep`'s content directly — it's fully chained away.
        assert_eq!(
            content_availability
                .iter()
                .map(|b| b.count_ones())
                .sum::<u32>(),
            0
        );
    }
}
