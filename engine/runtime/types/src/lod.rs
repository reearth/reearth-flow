use std::ops::{BitAnd, BitAndAssign, BitOrAssign};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct LodMask(
    u8, // lods bit mask
);

impl LodMask {
    pub fn all() -> Self {
        Self(0b11111)
    }

    pub fn add_lod(&mut self, lod_no: u8) {
        self.0 |= 1 << lod_no;
    }

    pub fn remove_lod(&mut self, lod_no: u8) {
        self.0 &= !(1 << lod_no);
    }

    pub fn has_lod(&self, lod_no: u8) -> bool {
        self.0 & (1 << lod_no) != 0
    }

    /// Returns the highest LOD number.
    ///
    /// It returns `None` if none of the LODs are set.
    pub fn highest_lod(&self) -> Option<u8> {
        match self.0 {
            0 => None,
            _ => Some(7 - self.0.leading_zeros() as u8),
        }
    }

    /// Returns the lowest LOD number.
    ///
    /// It returns `None` if none of the LODs are set.
    pub fn lowest_lod(&self) -> Option<u8> {
        match self.0 {
            0 => None,
            _ => Some(self.0.trailing_zeros() as u8),
        }
    }

    pub fn find_lods_by_citygml_value(value: &nusamai_citygml::Value) -> Self {
        find_lods(value)
    }
}

fn find_lods(value: &nusamai_citygml::Value) -> LodMask {
    let mut mask = LodMask::default();
    match value {
        nusamai_citygml::Value::Object(obj) => {
            if let nusamai_citygml::object::ObjectStereotype::Feature { geometries, .. } =
                &obj.stereotype
            {
                geometries.iter().for_each(|geom| mask.add_lod(geom.lod));
            }
            for value in obj.attributes.values() {
                mask |= find_lods(value);
            }
        }
        nusamai_citygml::Value::Array(arr) => {
            arr.iter().for_each(|value| mask |= find_lods(value));
        }
        _ => {}
    }
    mask
}

impl BitOrAssign for LodMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAndAssign for LodMask {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitAnd for LodMask {
    type Output = LodMask;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_mask() {
        let mut mask = LodMask::default();
        assert_eq!(mask.lowest_lod(), None);
        assert_eq!(mask.highest_lod(), None);

        mask.add_lod(1);
        assert_eq!(mask.lowest_lod(), Some(1));
        assert_eq!(mask.highest_lod(), Some(1));
        assert!(!mask.has_lod(0));

        mask.add_lod(2);
        assert_eq!(mask.lowest_lod(), Some(1));
        assert_eq!(mask.highest_lod(), Some(2));
        assert!(!mask.has_lod(3));

        mask.add_lod(3);
        assert_eq!(mask.lowest_lod(), Some(1));
        assert_eq!(mask.highest_lod(), Some(3));
        assert!(mask.has_lod(3));

        // bitand
        let mut mask2 = LodMask::default();
        mask2.add_lod(3);
        assert!((mask & mask2).has_lod(3));
        assert!(!(mask & mask2).has_lod(1));
    }

    #[test]
    fn test_add_lod() {
        let mut mask = LodMask::default();
        assert!((0..=3).all(|lod| !mask.has_lod(lod)));

        mask.add_lod(0);
        assert!(mask.has_lod(0));
        assert!((1..=3).all(|lod| !mask.has_lod(lod)));

        mask.add_lod(2);
        assert!([0, 2].iter().all(|&lod| mask.has_lod(lod)));
        assert!([1, 3].iter().all(|&lod| !mask.has_lod(lod)));
    }

    #[test]
    fn test_remove_lod() {
        let mut mask = LodMask::all();
        assert!((0..=3).all(|lod| mask.has_lod(lod)));

        mask.remove_lod(3);
        assert!(!mask.has_lod(3));
        assert!((0..=2).all(|lod| mask.has_lod(lod)));

        mask.remove_lod(1);
        assert!([1, 3].iter().all(|&lod| !mask.has_lod(lod)));
        assert!([0, 2].iter().all(|&lod| mask.has_lod(lod)));
    }
}
