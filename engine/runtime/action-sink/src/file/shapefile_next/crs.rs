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

    fn prj_for(epsg: u16) -> String {
        let mut buffer = Vec::new();
        write_prj(&mut buffer, EpsgCode::new(epsg))
            .expect("the CRS is expected to have an ESRI WKT1 form");
        String::from_utf8(buffer).unwrap()
    }

    /// Whatever is written must name its CRS back, or a reader cannot recover it.
    #[test]
    fn every_definition_written_names_a_crs_back() {
        for epsg in [
            4326u16, 3857, 6668, 6697, 6669, 6677, 2229, 4979, 10162, 10174,
        ] {
            let written = prj_for(epsg);
            assert!(
                identify_epsg(&written).is_some(),
                "EPSG:{epsg} wrote a definition naming nothing back: {written}"
            );
        }
    }

    /// A 3D geographic CRS falls back to its horizontal counterpart, PROJ writing
    /// a name for the 3D form that it does not resolve on the way back in.
    #[test]
    fn a_three_dimensional_crs_writes_its_horizontal_counterpart() {
        assert_eq!(
            identify_epsg(&prj_for(4979)),
            Some(EpsgCode::new(4326)),
            "4979 is expected to fall back to 4326"
        );
    }

    /// A compound CRS falls back likewise, PROJ identifying the compound weakly
    /// even though each of its parts is an exact match.
    #[test]
    fn a_compound_crs_writes_its_horizontal_counterpart() {
        assert_eq!(
            identify_epsg(&prj_for(10162)),
            Some(EpsgCode::new(6669)),
            "10162 is expected to fall back to its horizontal CRS 6669"
        );
    }

    /// A CRS whose definition already names itself back is written unchanged.
    #[test]
    fn a_crs_that_round_trips_is_written_unchanged() {
        for epsg in [4326u16, 6677, 6697] {
            assert_eq!(
                identify_epsg(&prj_for(epsg)),
                Some(EpsgCode::new(epsg)),
                "EPSG:{epsg} is expected to be written as itself"
            );
        }
    }
}
