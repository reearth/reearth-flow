pub(crate) mod errors;
pub(crate) mod metadata;
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

pub use metadata::*;
pub use utils::*;
pub use writer::*;
