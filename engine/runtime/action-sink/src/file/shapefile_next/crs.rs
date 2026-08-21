//! The `.prj` sidecar naming a shapefile's CRS.

use std::collections::HashMap;
use std::io::Write;
use std::sync::OnceLock;

use parking_lot::RwLock;

use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::ops::{esri_wkt1, identify_epsg};

/// Write the ESRI WKT1 definition of `epsg`. Errors when PROJ has no such form
/// for it.
pub(super) fn write_prj(mut writer: impl Write, epsg: EpsgCode) -> Result<(), std::io::Error> {
    let definition = definition_for(epsg).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("no ESRI WKT1 definition for EPSG:{epsg}: {e}"),
        )
    })?;
    writer.write_all(definition.as_bytes())?;
    writer.flush()
}

/// The definitions settled on, by CRS.
fn definition_cache() -> &'static RwLock<HashMap<EpsgCode, Result<String, String>>> {
    static CACHE: OnceLock<RwLock<HashMap<EpsgCode, Result<String, String>>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// The definition to write for `epsg`: its own where PROJ identifies that back
/// to it, else its horizontal counterpart's.
fn definition_for(epsg: EpsgCode) -> Result<String, String> {
    if let Some(cached) = definition_cache().read().get(&epsg) {
        return cached.clone();
    }
    let definition = definition_for_uncached(epsg);
    definition_cache().write().insert(epsg, definition.clone());
    definition
}

/// [`definition_for`], asked of PROJ.
fn definition_for_uncached(epsg: EpsgCode) -> Result<String, String> {
    let definition = esri_wkt1(epsg).map_err(|e| e.to_string())?;
    if identify_epsg(&definition) == Some(epsg) {
        return Ok(definition);
    }

    let horizontal = CoordinateFrame::Crs(epsg)
        .demote_to_2d()
        .map_err(|e| e.to_string())?;
    let CoordinateFrame::Crs(horizontal) = horizontal else {
        return Err(format!("EPSG:{epsg} has no horizontal counterpart"));
    };
    let alternative = esri_wkt1(horizontal).map_err(|e| e.to_string())?;
    if identify_epsg(&alternative) != Some(horizontal) {
        return Err(format!(
            "neither EPSG:{epsg} nor its horizontal counterpart EPSG:{horizontal} \
             has an ESRI WKT1 definition naming it back"
        ));
    }
    tracing::warn!(
        "EPSG:{epsg} has no ESRI WKT1 definition that names it back; writing its \
         horizontal counterpart EPSG:{horizontal}, which drops the vertical \
         reference the elevations are measured against"
    );
    Ok(alternative)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// What is written names the CRS back, or its horizontal counterpart for a
    /// 3D geographic or compound CRS.
    #[test]
    fn each_definition_names_its_crs_or_its_horizontal_counterpart_back() {
        for (epsg, expected) in [
            (4326u16, 4326u16),
            (3857, 3857),
            (6668, 6668),
            (6677, 6677),
            (6697, 6697),
            (2229, 2229),
            (4979, 4326),
            (10162, 6669),
        ] {
            let mut buffer = Vec::new();
            write_prj(&mut buffer, EpsgCode::new(epsg))
                .expect("the CRS is expected to have an ESRI WKT1 form");
            let written = String::from_utf8(buffer).unwrap();
            assert_eq!(
                identify_epsg(&written),
                Some(EpsgCode::new(expected)),
                "EPSG:{epsg} wrote {written}"
            );
        }
    }
}
