use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct VectorLayer {
    pub(crate) id: String,
    pub(crate) fields: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct TileMetadata {
    tilejson: String,
    tiles: Vec<String>,
    vector_layers: Vec<VectorLayer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    minzoom: u8,
    maxzoom: u8,
    bounds: [f64; 4],
    #[serde(skip_serializing_if = "Option::is_none")]
    center: Option<[f64; 3]>,
}

impl TileMetadata {
    pub(crate) fn from_tile_content(
        name: String,
        min_zoom: u8,
        max_zoom: u8,
        tile_content: &TileContent,
        tiles: Vec<String>,
        vector_layers: Vec<VectorLayer>,
    ) -> Self {
        let bounds = [
            tile_content.min_lng,
            tile_content.min_lat,
            tile_content.max_lng,
            tile_content.max_lat,
        ];
        let center = [
            (tile_content.min_lng + tile_content.max_lng) / 2.0,
            (tile_content.min_lat + tile_content.max_lat) / 2.0,
            max_zoom as f64,
        ];
        TileMetadata {
            tilejson: "3.0.0".to_string(),
            tiles,
            vector_layers,
            name: Some(name),
            description: None,
            version: None,
            minzoom: min_zoom,
            maxzoom: max_zoom,
            bounds,
            center: Some(center),
        }
    }
}

#[derive(Debug)]
pub(crate) struct TileContent {
    pub(crate) min_lng: f64,
    pub(crate) max_lng: f64,
    pub(crate) min_lat: f64,
    pub(crate) max_lat: f64,
}

impl Default for TileContent {
    fn default() -> Self {
        TileContent {
            min_lng: f64::MAX,
            max_lng: f64::MIN,
            min_lat: f64::MAX,
            max_lat: f64::MIN,
        }
    }
}
