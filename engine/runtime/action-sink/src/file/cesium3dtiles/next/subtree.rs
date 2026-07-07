//! `.subtree` binary encoding (3D Tiles 1.1 implicit tiling): the
//! `tileAvailability` / `contentAvailability` / `childSubtreeAvailability`
//! bitstreams telling a client which cells exist, without listing them in
//! JSON. Scoped to a dataset that fits in one subtree file:
//! `childSubtreeAvailability` is always `{"constant": 0}`, so chaining
//! `.subtree` files across a subtree boundary is unimplemented.

use std::collections::BTreeSet;

use serde_json::json;

use super::quadtree::Cell;

const MAGIC: &[u8; 4] = b"subt";
const VERSION: u32 = 1;

/// Build one `.subtree` file covering every level from the dataset root
/// (level 0) through `subtree_levels - 1`, given the set of cells that
/// actually hold content.
pub(super) fn build(occupied: &BTreeSet<Cell>, subtree_levels: u32) -> Vec<u8> {
    let total_tiles = level_offset(subtree_levels) as usize;
    let mut tile_availability = BitSet::new(total_tiles);
    let mut content_availability = BitSet::new(total_tiles);

    for &cell in occupied {
        content_availability.set(bit_index(cell));
        let mut ancestor = Some(cell);
        while let Some(c) = ancestor {
            tile_availability.set(bit_index(c));
            ancestor = c.parent();
        }
    }

    let tile_available_count = tile_availability.count_ones();
    let content_available_count = occupied.len();
    let tile_bytes = tile_availability.into_bytes();
    let content_bytes = content_availability.into_bytes();

    let json = json!({
        "buffers": [{"byteLength": tile_bytes.len() + content_bytes.len()}],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": tile_bytes.len()},
            {"buffer": 0, "byteOffset": tile_bytes.len(), "byteLength": content_bytes.len()},
        ],
        "tileAvailability": {"bitstream": 0, "availableCount": tile_available_count},
        "contentAvailability": [{"bitstream": 1, "availableCount": content_available_count}],
        "childSubtreeAvailability": {"constant": 0},
    });

    let mut json_bytes = serde_json::to_vec(&json).expect("subtree JSON is always serializable");
    while !json_bytes.len().is_multiple_of(8) {
        json_bytes.push(b' ');
    }

    let mut binary = tile_bytes;
    binary.extend(content_bytes);
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

/// Number of tiles across levels `0..level` of a complete quadtree — the bit
/// offset where `level`'s span starts in the availability bitstream.
fn level_offset(level: u32) -> u64 {
    (4u64.pow(level) - 1) / 3
}

/// A cell's bit index in the availability bitstream: its level's offset plus
/// its Morton index within that level (3D Tiles 1.1's fixed ordering).
fn bit_index(cell: Cell) -> u64 {
    level_offset(cell.level) + morton2d(cell.x, cell.y)
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

    #[test]
    fn header_lengths_match_actual_chunk_sizes() {
        let occupied = BTreeSet::from([Cell {
            level: 1,
            x: 0,
            y: 0,
        }]);
        let bytes = build(&occupied, 2);
        let json_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
        let binary_len = u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as usize;
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
        let bytes = build(&occupied, 2);
        let json_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
        let binary = &bytes[24 + json_len..];
        let tile_availability_len = level_offset(2).div_ceil(8) as usize;
        let tile_availability = &binary[..tile_availability_len];
        let content_availability = &binary[tile_availability_len..];

        let bit = |bytes: &[u8], i: u64| (bytes[(i / 8) as usize] >> (i % 8)) & 1;

        let root_idx = bit_index(Cell {
            level: 0,
            x: 0,
            y: 0,
        });
        let placed_idx = bit_index(Cell {
            level: 1,
            x: 1,
            y: 0,
        });
        let sibling_idx = bit_index(Cell {
            level: 1,
            x: 0,
            y: 0,
        });

        assert_eq!(bit(tile_availability, root_idx), 1);
        assert_eq!(bit(tile_availability, placed_idx), 1);
        assert_eq!(bit(tile_availability, sibling_idx), 0);

        assert_eq!(bit(content_availability, root_idx), 0);
        assert_eq!(bit(content_availability, placed_idx), 1);
    }
}
