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
    /// The code page the `.dbf` header states, for a table nothing else names an
    /// encoding for.
    Declared,
    /// UTF-8 and its aliases.
    Utf8,
    /// Any label `encoding_rs` recognises.
    Named(&'static encoding_rs::Encoding),
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
            .map(Self::Named)
            .ok_or_else(|| ShapefileError::UnsupportedEncoding(name.to_string()))
    }

    /// The encoding's name, for reporting.
    fn name(&self) -> &str {
        match self {
            Self::Declared => "the code page it declares",
            Self::Utf8 => "UTF-8",
            Self::Named(encoding) => encoding.name(),
        }
    }
}

/// The files of one shapefile, as read out of the archive.
#[derive(Default)]
struct Components {
    /// The stem as the archive spells it, for reporting; components are grouped
    /// under a case-folded stem, which is nobody's file name.
    name: String,
    shp: Option<Vec<u8>>,
    dbf: Option<Vec<u8>>,
    shx: Option<Vec<u8>>,
    prj: Option<Vec<u8>>,
    cpg: Option<Vec<u8>>,
}

impl Components {
    /// An empty group for the shapefile the archive spells `name`.
    fn named(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Whether both files a shapefile cannot be read without are present. The
    /// format also calls for a `.shx`, which this reader does without.
    fn is_complete(&self) -> bool {
        self.shp.is_some() && self.dbf.is_some()
    }
}

/// A shapefile opened for reading: its records, and what they are expressed in.
pub(super) struct Archive {
    reader: shapefile::Reader<Cursor<Vec<u8>>, Cursor<Vec<u8>>>,
    /// The CRS the `.prj` names, or `None` when there is none to name it.
    pub(super) epsg: Option<EpsgCode>,
    /// The attribute table's field names, in the order the table declares them.
    pub(super) field_names: Vec<String>,
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

    let encoding = resolve_encoding(
        encoding,
        components.cpg.as_deref(),
        components.dbf.as_deref(),
    )?;
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
    let field_names = table
        .fields()
        .iter()
        .map(|field| field.name().to_string())
        .collect();

    Ok(Archive {
        reader: shapefile::Reader::new(shapes, table),
        epsg,
        field_names,
    })
}

/// The files of the archive's shapefile, keyed by extension.
///
/// An archive holding more than one shapefile is read as the first by name, so
/// that the same archive always yields the same shapefile.
fn extract(content: &Bytes) -> Result<Components, SourceError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(content.as_ref()))
        .map_err(|e| SourceError::shapefile_reader(format!("Failed to read ZIP archive: {e}")))?;

    // Keyed by the case-folded stem, a component's name being spelled in whatever
    // case its producer chose, and ordered so that an archive of several
    // shapefiles resolves to the same one on every read.
    let mut groups: BTreeMap<String, Components> = BTreeMap::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            SourceError::shapefile_reader(format!("Failed to read ZIP entry at index {i}: {e}"))
        })?;
        let path = file.name().to_string();
        if file.is_dir() || is_metadata(&path) {
            continue;
        }
        let (stem, extension) = match classify(&path) {
            Entry::Component(stem, extension) => (stem, extension),
            Entry::Sidecar => {
                tracing::debug!("passing over '{path}', a shapefile sidecar holding no features");
                continue;
            }
            Entry::Unrelated => {
                tracing::debug!("passing over '{path}', which is no part of a shapefile");
                continue;
            }
        };

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| {
            SourceError::shapefile_reader(format!("Failed to read ZIP entry '{path}': {e}"))
        })?;

        let components = groups
            .entry(stem.to_lowercase())
            .or_insert_with(|| Components::named(&stem));
        match extension.as_str() {
            "shp" => components.shp = Some(buffer),
            "dbf" => components.dbf = Some(buffer),
            "shx" => components.shx = Some(buffer),
            "prj" => components.prj = Some(buffer),
            "cpg" => components.cpg = Some(buffer),
            _ => unreachable!("classify only yields the extensions above"),
        }
    }

    let complete: Vec<String> = groups
        .iter()
        .filter(|(_, c)| c.is_complete())
        .map(|(key, _)| key.clone())
        .collect();
    let Some(key) = complete.first() else {
        return Err(ShapefileError::NoCompleteShapefile.into());
    };
    let name = groups[key].name.clone();
    if complete.len() > 1 {
        let names: Vec<&str> = complete.iter().map(|k| groups[k].name.as_str()).collect();
        tracing::warn!(
            "the archive holds {} shapefiles ({}); reading '{name}' and ignoring the rest",
            complete.len(),
            names.join(", ")
        );
    }
    tracing::info!("reading the shapefile '{name}'");
    let key = key.clone();
    Ok(groups.remove(&key).expect("the key came from the map"))
}

/// Whether a ZIP entry is archive metadata rather than part of a shapefile.
fn is_metadata(path: &str) -> bool {
    path.contains("__MACOSX")
        || path
            .rsplit('/')
            .next()
            .is_some_and(|name| name.starts_with('.'))
}

/// What a ZIP entry is, as far as reading a shapefile goes.
#[derive(Debug, PartialEq, Eq)]
enum Entry {
    /// A file this reader takes something from: the shapefile's stem, then the
    /// component's extension in lower case.
    Component(String, String),
    /// A file a shapefile ships alongside its components that holds no feature
    /// data, so there is nothing to read from it.
    Sidecar,
    /// A file that is no part of a shapefile.
    Unrelated,
}

/// Extensions of the components this reader takes something from.
const COMPONENTS: [&str; 5] = ["shp", "dbf", "shx", "prj", "cpg"];

/// Extensions of the sidecars a shapefile ships with: derived spatial and
/// attribute indices, every one of them rebuildable from the components.
///
/// Recognised so that an entry passed over deliberately can be told apart from one
/// the reader does not know about. `.shp.xml`, the metadata document, is recognised
/// by its double extension instead.
const SIDECARS: [&str; 10] = [
    "sbn", "sbx", "fbn", "fbx", "ain", "aih", "atx", "ixs", "mxs", "qix",
];

/// What the ZIP entry `path` is.
///
/// A component's stem keeps the entry's directory, so shapefiles of the same name
/// in different directories stay apart.
fn classify(path: &str) -> Entry {
    let Some((stem, extension)) = path.rsplit_once('.') else {
        return Entry::Unrelated;
    };
    let extension = extension.to_lowercase();
    if COMPONENTS.contains(&extension.as_str()) {
        return Entry::Component(stem.to_string(), extension);
    }
    // The metadata document is named after the whole `.shp`, so its stem is a
    // shapefile's name with an extension still on it.
    if extension == "xml" && stem.to_lowercase().ends_with(".shp") {
        return Entry::Sidecar;
    }
    if SIDECARS.contains(&extension.as_str()) {
        return Entry::Sidecar;
    }
    Entry::Unrelated
}

/// The encoding to read the attribute table in: the parameter, else the `.cpg`,
/// else the code page its own header declares, else UTF-8.
fn resolve_encoding(
    parameter: &Option<String>,
    cpg: Option<&[u8]>,
    dbf: Option<&[u8]>,
) -> Result<Encoding, ShapefileError> {
    if let Some(name) = parameter.as_deref().filter(|name| !name.is_empty()) {
        tracing::debug!("reading the attribute table as {name}, from the parameter");
        return Encoding::from_name(name);
    }
    if let Some(name) = cpg.and_then(parse_cpg_encoding) {
        tracing::debug!("reading the attribute table as {name}, from the .cpg");
        return Encoding::from_name(&name);
    }
    match dbf.map(declared_code_page) {
        Some(DeclaredCodePage::Decodable) => {
            tracing::debug!("reading the attribute table in the code page its header declares");
            return Ok(Encoding::Declared);
        }
        Some(DeclaredCodePage::Undecodable(mark)) => tracing::warn!(
            "the attribute table declares the code page {mark:?}, which this build has \
             no decoder for; reading its text as UTF-8. Name an encoding in the \
             parameter to read it as something else"
        ),
        Some(DeclaredCodePage::Unstated) | None => {}
    }
    tracing::debug!("reading the attribute table as UTF-8, no encoding being stated");
    Ok(Encoding::Utf8)
}

/// Offset in a `.dbf` header of the byte stating the code page its text is written
/// in.
const CODE_PAGE_MARK_OFFSET: usize = 29;

/// What a `.dbf` header's code page mark amounts to.
#[derive(Debug, PartialEq, Eq)]
enum DeclaredCodePage {
    /// A code page this build can decode.
    Decodable,
    /// A code page this build has no decoder for.
    Undecodable(shapefile::dbase::CodePageMark),
    /// A header naming no code page.
    Unstated,
}

/// The code page a `.dbf` header declares.
///
/// Only the pages the web encodings cover can be decoded here; the DOS pages they
/// leave out are reported rather than delegated, `dbase` failing outright on a mark
/// it cannot resolve.
fn declared_code_page(dbf: &[u8]) -> DeclaredCodePage {
    use shapefile::dbase::CodePageMark::*;

    let Some(&code) = dbf.get(CODE_PAGE_MARK_OFFSET) else {
        return DeclaredCodePage::Unstated;
    };
    match shapefile::dbase::CodePageMark::from(code) {
        Undefined | Invalid => DeclaredCodePage::Unstated,
        CP1252 | CP866 | CP874 | CP1250 | CP1251 | CP1253 | CP1254 | CP1255 | CP1256 | CP932
        | CP936 | CP949 | CP950 | Utf8 => DeclaredCodePage::Decodable,
        undecodable => DeclaredCodePage::Undecodable(undecodable),
    }
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
    let wkt = wkt.trim().trim_start_matches('\u{feff}').trim_start();
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
///
/// [`Encoding::Declared`] leaves the choice to `dbase`, which takes it from the
/// header's code page mark. Only a mark [`declared_code_page`] found decodable
/// reaches here, so the mark cannot leave `dbase` without an encoding.
fn dbase_reader<T: Read + std::io::Seek>(
    source: T,
    encoding: Encoding,
) -> Result<shapefile::dbase::Reader<T>, SourceError> {
    tracing::debug!("reading the attribute table as {}", encoding.name());
    match encoding {
        Encoding::Declared => shapefile::dbase::Reader::new(source).map_err(|e| {
            SourceError::shapefile_reader(format!(
                "Failed to create dbase reader with the declared code page: {e}"
            ))
        }),
        Encoding::Utf8 => shapefile::dbase::Reader::new_with_encoding(
            source,
            shapefile::dbase::encoding::UnicodeLossy,
        )
        .map_err(|e| {
            SourceError::shapefile_reader(format!(
                "Failed to create dbase reader with UTF-8 encoding: {e}"
            ))
        }),
        Encoding::Named(encoding) => shapefile::dbase::Reader::new_with_encoding(
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

    fn component(path: &str) -> Option<(String, String)> {
        match classify(path) {
            Entry::Component(stem, extension) => Some((stem, extension)),
            _ => None,
        }
    }

    #[test]
    fn a_component_is_recognised_by_its_extension_whatever_its_case() {
        assert_eq!(
            component("dir/mesh3.SHP"),
            Some(("dir/mesh3".to_string(), "shp".to_string()))
        );
        assert_eq!(classify("readme.txt"), Entry::Unrelated);
        assert_eq!(classify("no-extension"), Entry::Unrelated);
    }

    // Two shapefiles of the same name in different directories are different
    // shapefiles, so their components must not be mixed.
    #[test]
    fn a_stem_keeps_its_directory() {
        assert_eq!(component("a/mesh3.shp").unwrap().0, "a/mesh3".to_string());
        assert_ne!(
            component("a/mesh3.shp").unwrap().0,
            component("b/mesh3.shp").unwrap().0
        );
    }

    // The indices and metadata a shapefile ships with hold no feature data, and are
    // recognised so an entry passed over deliberately is not mistaken for one the
    // reader does not know about.
    #[test]
    fn a_sidecar_is_recognised_as_one() {
        for path in [
            "mesh3.sbn",
            "mesh3.sbx",
            "mesh3.fbn",
            "mesh3.fbx",
            "mesh3.ain",
            "mesh3.aih",
            "mesh3.atx",
            "mesh3.ixs",
            "mesh3.mxs",
            "mesh3.qix",
            "mesh3.QIX",
        ] {
            assert_eq!(classify(path), Entry::Sidecar, "{path}");
        }
    }

    // The metadata document is named after the whole `.shp`, so it must not be taken
    // for a component of a shapefile called `mesh3.shp`.
    #[test]
    fn the_metadata_document_is_a_sidecar_not_a_component() {
        assert_eq!(classify("dir/mesh3.shp.xml"), Entry::Sidecar);
        assert_eq!(classify("dir/mesh3.SHP.XML"), Entry::Sidecar);
        // Some other XML in the archive is nothing to do with a shapefile.
        assert_eq!(classify("dir/metadata.xml"), Entry::Unrelated);
    }

    #[test]
    fn archive_metadata_is_not_a_component() {
        assert!(is_metadata("__MACOSX/._mesh3.shp"));
        assert!(is_metadata(".DS_Store"));
        assert!(is_metadata("dir/.hidden.shp"));
        assert!(!is_metadata("dir/mesh3.shp"));
    }

    /// A ZIP archive holding one entry per `(name, bytes)`.
    fn zipped(entries: &[(&str, &[u8])]) -> Bytes {
        use std::io::Write as _;

        let mut buffer = Vec::new();
        {
            let mut writer = zip::ZipWriter::new(Cursor::new(&mut buffer));
            for (name, bytes) in entries {
                writer
                    .start_file(*name, zip::write::SimpleFileOptions::default())
                    .expect("the entry is expected to start");
                writer
                    .write_all(bytes)
                    .expect("the entry is expected write");
            }
            writer.finish().expect("the archive is expected to finish");
        }
        Bytes::from(buffer)
    }

    // A producer spells the stem in whatever case it likes, and its components need
    // not agree with each other, so pairing them cannot turn on it.
    #[test]
    fn components_pair_whatever_case_their_stem_is_spelled_in() {
        let content = zipped(&[("DATA.SHP", b"shp"), ("data.dbf", b"dbf")]);
        let components = extract(&content).expect("the components are expected to pair");
        assert_eq!(components.shp.as_deref(), Some(b"shp".as_ref()));
        assert_eq!(components.dbf.as_deref(), Some(b"dbf".as_ref()));
    }

    // A sidecar carries no feature data, so it can neither complete a shapefile nor
    // stand in for a component that is missing.
    #[test]
    fn a_sidecar_cannot_complete_a_shapefile() {
        let content = zipped(&[("data.shp", b"shp"), ("data.qix", b"index")]);
        assert!(extract(&content).is_err());
    }

    // `data.shp.xml` must not be read as a component of a shapefile whose stem is
    // `data.shp`, which would make the metadata document its own shapefile.
    #[test]
    fn the_metadata_document_does_not_become_a_shapefile_of_its_own() {
        let content = zipped(&[
            ("data.shp", b"shp"),
            ("data.dbf", b"dbf"),
            ("data.shp.xml", b"<metadata/>"),
        ]);
        let components = extract(&content).expect("the components are expected to pair");
        assert_eq!(components.name, "data");
    }

    /// A `.dbf` header whose code page byte is `code`.
    fn dbf_declaring(code: u8) -> Vec<u8> {
        let mut header = vec![0u8; 32];
        header[CODE_PAGE_MARK_OFFSET] = code;
        header
    }

    #[test]
    fn the_parameter_overrides_the_cpg() {
        let encoding =
            resolve_encoding(&Some("Shift_JIS".to_string()), Some(b"UTF-8"), None).unwrap();
        assert_eq!(encoding.name(), "Shift_JIS");
    }

    #[test]
    fn an_unusable_encoding_is_rejected() {
        assert!(resolve_encoding(&Some("UTF-16".to_string()), None, None).is_err());
        assert!(resolve_encoding(&Some("not-an-encoding".to_string()), None, None).is_err());
    }

    // A `.cpg` states the encoding outright, so it is taken over the header's code
    // page mark.
    #[test]
    fn the_cpg_overrides_the_declared_code_page() {
        let encoding = resolve_encoding(&None, Some(b"UTF-8"), Some(&dbf_declaring(0x7B))).unwrap();
        assert_eq!(encoding, Encoding::Utf8);
    }

    // A table that names its own code page must be read in it. Reading it as UTF-8
    // instead garbles every non-ASCII field, which is what a Shift_JIS table
    // shipped without a `.cpg` used to come back as.
    #[test]
    fn a_declared_code_page_is_read_in_when_nothing_else_names_one() {
        let encoding = resolve_encoding(&None, None, Some(&dbf_declaring(0x7B))).unwrap();
        assert_eq!(encoding, Encoding::Declared);
    }

    #[test]
    fn a_table_naming_no_code_page_is_read_as_utf8() {
        let encoding = resolve_encoding(&None, None, Some(&dbf_declaring(0x00))).unwrap();
        assert_eq!(encoding, Encoding::Utf8);
        assert_eq!(resolve_encoding(&None, None, None).unwrap(), Encoding::Utf8);
    }

    // `dbase` fails outright on a code page it cannot resolve, so a DOS page the web
    // encodings leave out must not be handed to it.
    #[test]
    fn a_code_page_with_no_decoder_falls_back_rather_than_failing() {
        // 0x01 is code page 437, which the web encodings do not cover.
        let encoding = resolve_encoding(&None, None, Some(&dbf_declaring(0x01))).unwrap();
        assert_eq!(encoding, Encoding::Utf8);
        assert!(matches!(
            declared_code_page(&dbf_declaring(0x01)),
            DeclaredCodePage::Undecodable(_)
        ));
    }

    // A header too short to hold a code page byte states nothing.
    #[test]
    fn a_truncated_header_states_no_code_page() {
        assert_eq!(declared_code_page(&[0u8; 4]), DeclaredCodePage::Unstated);
    }

    // A projected CRS must not be reported as the geographic CRS it is built on.
    #[test]
    fn a_projected_crs_is_not_taken_for_its_datum() {
        let prj = br#"GEOGCS["GCS_JGD_2011",DATUM["D_JGD_2011",SPHEROID["GRS_1980",6378137.0,298.257222101]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]]"#;
        assert_eq!(parse_prj_epsg(prj), Some(6668));
    }
}
