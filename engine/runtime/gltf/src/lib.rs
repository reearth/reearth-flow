pub(crate) mod errors;
pub(crate) mod geometry;
pub(crate) mod metadata;
pub(crate) mod reader;
pub(crate) mod utils;
pub(crate) mod writer;

#[derive(Debug, Clone)]
pub struct BoundingVolume {
    pub min_lng: f64,
    pub max_lng: f64,
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_height: f64,
    pub max_height: f64,
}

impl BoundingVolume {
    pub fn update(&mut self, other: &Self) {
        self.min_lng = self.min_lng.min(other.min_lng);
        self.max_lng = self.max_lng.max(other.max_lng);
        self.min_lat = self.min_lat.min(other.min_lat);
        self.max_lat = self.max_lat.max(other.max_lat);
        self.min_height = self.min_height.min(other.min_height);
        self.max_height = self.max_height.max(other.max_height);
    }
}

impl Default for BoundingVolume {
    fn default() -> Self {
        Self {
            min_lng: f64::MAX,
            max_lng: f64::MIN,
            min_lat: f64::MAX,
            max_lat: f64::MIN,
            min_height: f64::MAX,
            max_height: f64::MIN,
        }
    }
}

pub use geometry::*;
pub use metadata::*;
pub use reader::*;
pub use utils::*;
pub use writer::*;

#[cfg(test)]
pub(crate) mod test_utils {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    fn testdata_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata")
    }

    /// Lazy loader for test GLB files from testdata directory.
    /// Files are loaded on first access and cached in memory.
    ///
    /// Available files:
    /// - "minimal_rectangle.glb" - Minimal GLB with rectangle (0,0,0)-(1,1,0), NO EXT_structural_metadata
    /// - "test_data_39255_tran_AuxiliaryTrafficArea.glb" - GLB with EXT_structural_metadata extension
    pub fn load_testdata(filename: &str) -> Vec<u8> {
        static CACHE: OnceLock<Mutex<HashMap<String, Vec<u8>>>> = OnceLock::new();
        let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

        let mut cache = cache.lock().unwrap();
        cache
            .entry(filename.to_string())
            .or_insert_with(|| {
                std::fs::read(testdata_dir().join(filename))
                    .unwrap_or_else(|_| panic!("Failed to load testdata: {}", filename))
            })
            .clone()
    }
}
