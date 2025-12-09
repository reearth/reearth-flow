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
    /// Minimal GLB with rectangle (0,0,0)-(1,1,0) as two triangles, without EXT_structural_metadata
    /// Vertices: (0,0,0), (1,0,0), (1,1,0), (0,1,0)
    /// Triangles: [0,1,2] and [0,2,3]
    pub const MINIMAL_GLB_BASE64: &str = "Z2xURgIAAAA4AgAA4AEAAEpTT057ImFzc2V0Ijp7InZlcnNpb24iOiIyLjAifSwic2NlbmUiOjAsInNjZW5lcyI6W3sibm9kZXMiOlswXX1dLCJub2RlcyI6W3sibWVzaCI6MH1dLCJtZXNoZXMiOlt7InByaW1pdGl2ZXMiOlt7ImF0dHJpYnV0ZXMiOnsiUE9TSVRJT04iOjB9LCJpbmRpY2VzIjoxLCJtb2RlIjo0fV19XSwiYWNjZXNzb3JzIjpbeyJidWZmZXJWaWV3IjowLCJjb21wb25lbnRUeXBlIjo1MTI2LCJjb3VudCI6NCwidHlwZSI6IlZFQzMiLCJtaW4iOlswLjAsMC4wLDAuMF0sIm1heCI6WzEuMCwxLjAsMC4wXX0seyJidWZmZXJWaWV3IjoxLCJjb21wb25lbnRUeXBlIjo1MTIzLCJjb3VudCI6NiwidHlwZSI6IlNDQUxBUiJ9XSwiYnVmZmVyVmlld3MiOlt7ImJ1ZmZlciI6MCwiYnl0ZU9mZnNldCI6MCwiYnl0ZUxlbmd0aCI6NDh9LHsiYnVmZmVyIjowLCJieXRlT2Zmc2V0Ijo0OCwiYnl0ZUxlbmd0aCI6MTJ9XSwiYnVmZmVycyI6W3siYnl0ZUxlbmd0aCI6NjB9XX0gICA8AAAAQklOAAAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAAAgD8AAIA/AAAAAAAAAAAAAIA/AAAAAAAAAQACAAAAAgADAA==";
}
