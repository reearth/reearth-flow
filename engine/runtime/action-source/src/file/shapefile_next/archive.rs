//! The ZIP archive a shapefile arrives in: its components (`.shp`, `.shx`,
//! `.dbf`, `.prj`, `.cpg`) sharing one stem.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use bytes::Bytes;
use reearth_flow_geometry::coordinate::EpsgCode;

use super::record::Field;
use crate::errors::{ShapefileError, SourceError};

/// Character encoding of a `.dbf`'s text fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Encoding {
    /// The code page the `.dbf` header states.
    Declared,
    /// UTF-8 and its aliases.
    Utf8,
    /// Any label `encoding_rs` recognises.
    Named(&'static encoding_rs::Encoding),
}

impl Encoding {
    /// The encoding `name` labels. Errors on an unknown label and on UTF-16.
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
    /// The stem as the archive spells it.
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

    /// Whether the `.shp` and `.dbf` are both present.
    fn is_complete(&self) -> bool {
        self.shp.is_some() && self.dbf.is_some()
    }
}

/// A shapefile opened for reading.
pub(super) struct Archive {
    reader: shapefile::Reader<Cursor<Vec<u8>>, Cursor<Vec<u8>>>,
    /// The CRS the `.prj` names, if any.
    pub(super) epsg: Option<EpsgCode>,
    /// The attribute table's fields, in the order the table declares them.
    pub(super) fields: Vec<Field>,
}

impl Archive {
    /// The records, one at a time.
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

/// Open the shapefile inside the ZIP archive `content`, reading its table in
/// `encoding` when given. Errors when no complete shapefile is found or the
/// encoding is unusable.
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
    let dbf = components.dbf.expect("a complete shapefile has a .dbf");
    let decimal_places = decimal_places(&dbf);
    let dbf = Cursor::new(dbf);

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
    let fields = table
        .fields()
        .iter()
        .enumerate()
        .map(|(i, field)| Field {
            name: field.name().to_string(),
            integral: matches!(
                field.field_type(),
                shapefile::dbase::FieldType::Numeric | shapefile::dbase::FieldType::Float
            ) && decimal_places.get(i) == Some(&0),
        })
        .collect();

    Ok(Archive {
        reader: shapefile::Reader::new(shapes, table),
        epsg,
        fields,
    })
}

/// The components of the archive's shapefile, paired by case-folded stem. Of
/// several shapefiles, the first by name is taken.
fn extract(content: &Bytes) -> Result<Components, SourceError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(content.as_ref()))
        .map_err(|e| SourceError::shapefile_reader(format!("Failed to read ZIP archive: {e}")))?;

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

/// What a ZIP entry is to a shapefile.
#[derive(Debug, PartialEq, Eq)]
enum Entry {
    /// A component: the shapefile's stem and the lower-cased extension.
    Component(String, String),
    /// A derived index or metadata file holding no feature data.
    Sidecar,
    /// No part of a shapefile.
    Unrelated,
}

/// Extensions of the components read.
const COMPONENTS: [&str; 5] = ["shp", "dbf", "shx", "prj", "cpg"];

/// Extensions of the index sidecars; the `.shp.xml` metadata document is
/// recognised by its double extension instead.
const SIDECARS: [&str; 10] = [
    "sbn", "sbx", "fbn", "fbx", "ain", "aih", "atx", "ixs", "mxs", "qix",
];

/// What the ZIP entry `path` is; a component's stem keeps its directory.
fn classify(path: &str) -> Entry {
    let Some((stem, extension)) = path.rsplit_once('.') else {
        return Entry::Unrelated;
    };
    let extension = extension.to_lowercase();
    if COMPONENTS.contains(&extension.as_str()) {
        return Entry::Component(stem.to_string(), extension);
    }
    if extension == "xml" && stem.to_lowercase().ends_with(".shp") {
        return Entry::Sidecar;
    }
    if SIDECARS.contains(&extension.as_str()) {
        return Entry::Sidecar;
    }
    Entry::Unrelated
}

/// The encoding to read the table in: the parameter, else the `.cpg`, else the
/// header's code page, else UTF-8.
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

/// Offset in a `.dbf` header of the code page mark.
const CODE_PAGE_MARK_OFFSET: usize = 29;
/// Offset in a `.dbf` of the first field descriptor.
const FIELD_DESCRIPTORS_OFFSET: usize = 32;
/// Bytes of one field descriptor.
const FIELD_DESCRIPTOR_BYTES: usize = 32;
/// The byte that ends the field descriptors.
const FIELD_DESCRIPTORS_TERMINATOR: u8 = 0x0D;
/// Offset in a field descriptor of its decimal count.
const DECIMAL_COUNT_OFFSET: usize = 17;

/// The decimal count each field descriptor of a `.dbf` declares, in table order.
fn decimal_places(dbf: &[u8]) -> Vec<u8> {
    dbf.get(FIELD_DESCRIPTORS_OFFSET..)
        .unwrap_or(&[])
        .chunks_exact(FIELD_DESCRIPTOR_BYTES)
        .take_while(|descriptor| descriptor[0] != FIELD_DESCRIPTORS_TERMINATOR)
        .map(|descriptor| descriptor[DECIMAL_COUNT_OFFSET])
        .collect()
}

/// What a `.dbf` header's code page mark says.
#[derive(Debug, PartialEq, Eq)]
enum DeclaredCodePage {
    /// A code page this build can decode.
    Decodable,
    /// A code page this build has no decoder for.
    Undecodable(shapefile::dbase::CodePageMark),
    /// A header naming no code page.
    Unstated,
}

/// The code page a `.dbf` header declares; only the pages the web encodings
/// cover are decodable.
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

/// The encoding name a `.cpg` states on its first line.
fn parse_cpg_encoding(cpg: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(cpg);
    let name = text.lines().next()?.trim();
    (!name.is_empty()).then(|| name.to_string())
}

/// The EPSG code PROJ's database matches the `.prj` definition to, if any.
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

/// A `dbase` reader decoding text with `encoding`. [`Encoding::Declared`] must
/// only be passed for a mark [`declared_code_page`] found decodable.
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

    #[test]
    fn components_pair_whatever_case_their_stem_is_spelled_in() {
        let content = zipped(&[("DATA.SHP", b"shp"), ("data.dbf", b"dbf")]);
        let components = extract(&content).expect("the components are expected to pair");
        assert_eq!(components.shp.as_deref(), Some(b"shp".as_ref()));
        assert_eq!(components.dbf.as_deref(), Some(b"dbf".as_ref()));
    }

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
    fn the_encoding_comes_from_the_parameter_then_the_cpg_then_the_header() {
        let shift_jis = Some("Shift_JIS".to_string());
        let declared = dbf_declaring(0x7B);
        assert_eq!(
            resolve_encoding(&shift_jis, Some(b"UTF-8"), Some(&declared))
                .unwrap()
                .name(),
            "Shift_JIS"
        );
        assert_eq!(
            resolve_encoding(&None, Some(b"UTF-8"), Some(&declared)).unwrap(),
            Encoding::Utf8
        );
        assert_eq!(
            resolve_encoding(&None, None, Some(&declared)).unwrap(),
            Encoding::Declared
        );
        assert_eq!(
            resolve_encoding(&None, None, Some(&dbf_declaring(0x00))).unwrap(),
            Encoding::Utf8
        );
    }

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

    #[test]
    fn each_field_descriptor_states_its_decimal_count() {
        use shapefile::dbase::{FieldName, TableWriterBuilder};

        let mut dbf = Vec::new();
        TableWriterBuilder::new()
            .add_numeric_field(FieldName::try_from("count").unwrap(), 10, 0)
            .add_character_field(FieldName::try_from("name").unwrap(), 20)
            .add_numeric_field(FieldName::try_from("ratio").unwrap(), 10, 3)
            .build_with_dest(Cursor::new(&mut dbf))
            .finalize()
            .expect("the table is expected to write");
        assert_eq!(decimal_places(&dbf), vec![0, 0, 3]);
    }

    #[test]
    fn a_projected_crs_is_not_taken_for_its_datum() {
        let prj = br#"GEOGCS["GCS_JGD_2011",DATUM["D_JGD_2011",SPHEROID["GRS_1980",6378137.0,298.257222101]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]]"#;
        assert_eq!(parse_prj_epsg(prj), Some(6668));
    }
}
