use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TileMetadata {
    name: String,
    description: String,
    version: u8,
    minzoom: u8,
    maxzoom: u8,
    bounds: String,
    center: String,
    #[serde(rename = "type")]
    r#type: String,
    format: String,
}

impl TileMetadata {
    pub(crate) fn from_tile_content(
        name: String,
        min_zoom: u8,
        max_zoom: u8,
        tile_content: &TileContent,
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
        ];
        TileMetadata {
            name,
            description: "".to_string(),
            version: 2,
            minzoom: min_zoom,
            maxzoom: max_zoom,
            bounds: bounds
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(","),
            center: center
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(","),
            r#type: "overlay".to_string(),
            format: "pbf".to_string(),
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
