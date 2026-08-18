//! The `.prj` sidecar describing what a shapefile's coordinates are expressed in.

use std::io::Write;

use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::ops::{esri_wkt1, identify_epsg};

/// Write the ESRI WKT1 definition of `epsg`, the dialect a `.prj` is read in.
///
/// The definition comes from PROJ's database. Errors when PROJ cannot resolve the
/// code or has no ESRI WKT1 form for it, which is the case for a geocentric CRS.
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

/// The definition to write for `epsg`: its own where that names it back, and its
/// horizontal counterpart's where it does not.
///
/// Not every CRS survives the trip out through ESRI WKT1 and back: PROJ writes a
/// name for a 3D geographic CRS that it does not itself resolve, and identifies a
/// compound CRS weakly even when both its parts are exact. Writing a definition
/// that no longer names the CRS would leave a reader to guess, so the horizontal
/// CRS is written instead, which does name itself back. The elevations are written
/// either way; what is given up is the vertical CRS they are measured against,
/// which a shapefile has nowhere to record.
fn definition_for(epsg: EpsgCode) -> Result<String, String> {
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

    /// Whatever is written must name a CRS back: itself where its definition
    /// does, its horizontal counterpart where PROJ writes a name it does not
    /// resolve (a 3D geographic CRS) or identifies weakly (a compound CRS).
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
