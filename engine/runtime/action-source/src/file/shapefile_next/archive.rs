//! The ZIP archive a shapefile arrives in, and the sidecars describing how to
//! read it.
//!
//! A shapefile is a set of files sharing a stem: the shapes (`.shp`), their index
//! (`.shx`), the attribute table (`.dbf`), the CRS (`.prj`) and the table's
//! encoding (`.cpg`).

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use bytes::Bytes;
use reearth_flow_geometry::coordinate::EpsgCode;

use crate::errors::{ShapefileError, SourceError};

/// Character encoding of a `.dbf`'s text fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Encoding {
    /// UTF-8 and its aliases.
    Utf8,
    /// Any label `encoding_rs` recognises.
    EncodingRs(&'static encoding_rs::Encoding),
}

impl Encoding {
    /// The encoding `name` labels.
    ///
    /// Errors on a label no encoding matches, and on UTF-16, which the `dbase`
    /// reader cannot decode.
    fn from_name(name: &str) -> Result<Self, ShapefileError> {
        let upper = name.to_uppercase();
        if matches!(upper.as_str(), "UTF-8" | "UTF8" | "UNICODE" | "UTF_8") {
            return Ok(Self::Utf8);
        }
        if matches!(
            upper.as_str(),
            "UTF-16" | "UTF16" | "UTF-16LE" | "UTF-16BE" | "UTF_16"
        ) {
            return Err(ShapefileError::Utf16NotSupported);
        }
        encoding_rs::Encoding::for_label(name.as_bytes())
            .map(Self::EncodingRs)
            .ok_or_else(|| ShapefileError::UnsupportedEncoding(name.to_string()))
    }

    /// The encoding's name, for reporting.
    fn name(&self) -> &str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::EncodingRs(encoding) => encoding.name(),
        }
    }
}

/// The files of one shapefile, as read out of the archive.
#[derive(Default)]
struct Components {
    shp: Option<Vec<u8>>,
    dbf: Option<Vec<u8>>,
    shx: Option<Vec<u8>>,
    prj: Option<Vec<u8>>,
    cpg: Option<Vec<u8>>,
}

impl Components {
    /// Whether both files a shapefile cannot be read without are present.
    fn is_complete(&self) -> bool {
        self.shp.is_some() && self.dbf.is_some()
    }
}

/// A shapefile opened for reading: its records, and what they are expressed in.
pub(super) struct Archive {
    reader: shapefile::Reader<Cursor<Vec<u8>>, Cursor<Vec<u8>>>,
    /// The CRS the `.prj` names, or `None` when there is none to name it.
    pub(super) epsg: Option<EpsgCode>,
}

impl Archive {
    /// Read the records, one at a time.
    pub(super) fn records(
        &mut self,
    ) -> impl Iterator<Item = Result<(shapefile::Shape, shapefile::dbase::Record), shapefile::Error>> + '_
    {
        self.reader.iter_shapes_and_records()
    }
}

/// Whether `content` starts with a ZIP local file header.
pub(super) fn is_zip(content: &Bytes) -> bool {
    content.starts_with(b"PK")
}

/// Open the shapefile inside the ZIP archive `content`.
///
/// `encoding` overrides the archive's `.cpg`. Errors when the archive holds no
/// complete shapefile, and when the encoding to read the table in is unusable.
pub(super) fn open(content: &Bytes, encoding: &Option<String>) -> Result<Archive, SourceError> {
    let components = extract(content)?;

    let encoding = resolve_encoding(encoding, components.cpg.as_deref())?;
    let epsg = components
        .prj
        .as_deref()
        .and_then(parse_prj_epsg)
        .map(EpsgCode::new);
    match epsg {
        Some(code) => tracing::info!("the shapefile's coordinates are in EPSG:{code}"),
        None => tracing::warn!(
            "the shapefile names no CRS its coordinates can be placed in; they will \
             carry none"
        ),
    }

    let shp = Cursor::new(components.shp.expect("a complete shapefile has a .shp"));
    let dbf = Cursor::new(components.dbf.expect("a complete shapefile has a .dbf"));

    // The index gives each record's offset, so the shapes can be read without
    // walking the whole file.
    let shapes = match components.shx {
        Some(shx) => shapefile::ShapeReader::with_shx(shp, Cursor::new(shx)).map_err(|e| {
            SourceError::shapefile_reader(format!("Failed to create shape reader with index: {e}"))
        })?,
        None => {
            tracing::warn!("the shapefile has no .shx index; reading its shapes in order");
            shapefile::ShapeReader::new(shp).map_err(|e| {
                SourceError::shapefile_reader(format!("Failed to create shape reader: {e}"))
            })?
        }
    };
    let table = dbase_reader(dbf, encoding)?;

    Ok(Archive {
        reader: shapefile::Reader::new(shapes, table),
        epsg,
    })
}

/// The files of the archive's shapefile, keyed by extension.
///
/// An archive holding more than one shapefile is read as the first by name, so
/// that the same archive always yields the same shapefile.
fn extract(content: &Bytes) -> Result<Components, SourceError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(content.as_ref()))
        .map_err(|e| SourceError::shapefile_reader(format!("Failed to read ZIP archive: {e}")))?;

    // Ordered so that an archive of several shapefiles resolves to the same one
    // on every read.
    let mut groups: BTreeMap<String, Components> = BTreeMap::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            SourceError::shapefile_reader(format!("Failed to read ZIP entry at index {i}: {e}"))
        })?;
        let path = file.name().to_string();
        if file.is_dir() || is_metadata(&path) {
            continue;
        }
        let Some((stem, extension)) = split_component(&path) else {
            continue;
        };

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| {
            SourceError::shapefile_reader(format!("Failed to read ZIP entry '{path}': {e}"))
        })?;

        let components = groups.entry(stem).or_default();
        match extension.as_str() {
            "shp" => components.shp = Some(buffer),
            "dbf" => components.dbf = Some(buffer),
            "shx" => components.shx = Some(buffer),
            "prj" => components.prj = Some(buffer),
            "cpg" => components.cpg = Some(buffer),
            _ => unreachable!("split_component only yields the extensions above"),
        }
    }

    let complete: Vec<String> = groups
        .iter()
        .filter(|(_, c)| c.is_complete())
        .map(|(stem, _)| stem.clone())
        .collect();
    let Some(stem) = complete.first() else {
        return Err(ShapefileError::NoCompleteShapefile.into());
    };
    if complete.len() > 1 {
        tracing::warn!(
            "the archive holds {} shapefiles ({}); reading '{stem}' and ignoring the rest",
            complete.len(),
            complete.join(", ")
        );
    }
    tracing::info!("reading the shapefile '{stem}'");
    Ok(groups.remove(stem).expect("the stem came from the map"))
}

/// Whether a ZIP entry is archive metadata rather than part of a shapefile.
fn is_metadata(path: &str) -> bool {
    path.contains("__MACOSX")
        || path
            .rsplit('/')
            .next()
            .is_some_and(|name| name.starts_with('.'))
}

/// A ZIP entry's `(stem, extension)` when it names a shapefile component, or
/// `None` for any other file.
///
/// The stem keeps the entry's directory, so shapefiles of the same name in
/// different directories stay apart.
fn split_component(path: &str) -> Option<(String, String)> {
    let (stem, extension) = path.rsplit_once('.')?;
    let extension = extension.to_lowercase();
    matches!(extension.as_str(), "shp" | "dbf" | "shx" | "prj" | "cpg")
        .then(|| (stem.to_string(), extension))
}

/// The encoding to read the attribute table in: the parameter, else the `.cpg`,
/// else UTF-8.
fn resolve_encoding(
    parameter: &Option<String>,
    cpg: Option<&[u8]>,
) -> Result<Encoding, ShapefileError> {
    if let Some(name) = parameter.as_deref().filter(|name| !name.is_empty()) {
        tracing::debug!("reading the attribute table as {name}, from the parameter");
        return Encoding::from_name(name);
    }
    if let Some(name) = cpg.and_then(parse_cpg_encoding) {
        tracing::debug!("reading the attribute table as {name}, from the .cpg");
        return Encoding::from_name(&name);
    }
    tracing::debug!("reading the attribute table as UTF-8, no encoding being stated");
    Ok(Encoding::Utf8)
}

/// The encoding name a `.cpg` states, which is its first line.
fn parse_cpg_encoding(cpg: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(cpg);
    let name = text.lines().next()?.trim();
    (!name.is_empty()).then(|| name.to_string())
}

/// The EPSG code the `.prj` names, or `None` when nothing in PROJ's database
/// matches it.
///
/// `.prj` holds a WKT CRS definition, usually in the ESRI dialect, which carries
/// no authority code of its own, so the definition is matched against the
/// database rather than its name being recognised here.
fn parse_prj_epsg(prj: &[u8]) -> Option<u16> {
    let wkt = String::from_utf8_lossy(prj);
    let wkt = wkt.trim();
    if wkt.is_empty() {
        return None;
    }
    match reearth_flow_geometry::ops::identify_epsg(wkt) {
        Some(code) => Some(code.get()),
        None => {
            tracing::warn!("no CRS in the PROJ database matches the .prj definition");
            None
        }
    }
}

/// A `dbase` reader decoding text with `encoding`.
fn dbase_reader<T: Read + std::io::Seek>(
    source: T,
    encoding: Encoding,
) -> Result<shapefile::dbase::Reader<T>, SourceError> {
    tracing::debug!("reading the attribute table as {}", encoding.name());
    match encoding {
        Encoding::Utf8 => shapefile::dbase::Reader::new_with_encoding(
            source,
            shapefile::dbase::encoding::UnicodeLossy,
        )
        .map_err(|e| {
            SourceError::shapefile_reader(format!(
                "Failed to create dbase reader with UTF-8 encoding: {e}"
            ))
        }),
        Encoding::EncodingRs(encoding) => shapefile::dbase::Reader::new_with_encoding(
            source,
            shapefile::dbase::encoding::EncodingRs::from(encoding),
        )
        .map_err(|e| {
            SourceError::shapefile_reader(format!(
                "Failed to create dbase reader with {} encoding: {e}",
                encoding.name()
            ))
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_component_is_recognised_by_its_extension_whatever_its_case() {
        assert_eq!(
            split_component("dir/mesh3.SHP"),
            Some(("dir/mesh3".to_string(), "shp".to_string()))
        );
        assert_eq!(split_component("readme.txt"), None);
        assert_eq!(split_component("no-extension"), None);
    }

    // Two shapefiles of the same name in different directories are different
    // shapefiles, so their components must not be mixed.
    #[test]
    fn a_stem_keeps_its_directory() {
        assert_eq!(
            split_component("a/mesh3.shp").unwrap().0,
            "a/mesh3".to_string()
        );
        assert_ne!(
            split_component("a/mesh3.shp").unwrap().0,
            split_component("b/mesh3.shp").unwrap().0
        );
    }

    #[test]
    fn archive_metadata_is_not_a_component() {
        assert!(is_metadata("__MACOSX/._mesh3.shp"));
        assert!(is_metadata(".DS_Store"));
        assert!(is_metadata("dir/.hidden.shp"));
        assert!(!is_metadata("dir/mesh3.shp"));
    }

    #[test]
    fn the_parameter_overrides_the_cpg() {
        let encoding = resolve_encoding(&Some("Shift_JIS".to_string()), Some(b"UTF-8")).unwrap();
        assert_eq!(encoding.name(), "Shift_JIS");
    }

    #[test]
    fn an_unusable_encoding_is_rejected() {
        assert!(resolve_encoding(&Some("UTF-16".to_string()), None).is_err());
        assert!(resolve_encoding(&Some("not-an-encoding".to_string()), None).is_err());
    }

    // A projected CRS must not be reported as the geographic CRS it is built on.
    #[test]
    fn a_projected_crs_is_not_taken_for_its_datum() {
        let prj = br#"GEOGCS["GCS_JGD_2011",DATUM["D_JGD_2011",SPHEROID["GRS_1980",6378137.0,298.257222101]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]]"#;
        assert_eq!(parse_prj_epsg(prj), Some(6668));
    }
}
