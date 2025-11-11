use thiserror::Error;

#[derive(Error, Debug)]
pub enum SourceError {
    #[error("CityGmlReaderFactory error: {0}")]
    CityGmlReaderFactory(String),
    #[error("CityGmlFileReader error: {0}")]
    CityGmlFileReader(String),
    #[error("CsvReaderFactory error: {0}")]
    CsvReaderFactory(String),
    #[error("CsvFileReader error: {0}")]
    CsvFileReader(String),
    #[error("JsonReaderFactory error: {0}")]
    JsonReaderFactory(String),
    #[error("JsonFileReader error: {0}")]
    JsonFileReader(String),
    #[error("GeoJsonReaderFactory error: {0}")]
    GeoJsonReaderFactory(String),
    #[error("GeoJsonFileReader error: {0}")]
    GeoJsonFileReader(String),
    #[error("ShapefileReaderFactory error: {0}")]
    ShapefileReaderFactory(String),
    #[error("ShapefileReader error: {0}")]
    ShapefileReader(String),
    #[error("CzmlReaderFactory error: {0}")]
    CzmlReaderFactory(String),
    #[error("CzmlReader error: {0}")]
    CzmlReader(String),
    #[error("GeoPackageReader error: {0}")]
    GeoPackageReader(String),
    #[error("GltfReaderFactory error: {0}")]
    GltfReaderFactory(String),
    #[error("GltfReader error: {0}")]
    GltfReader(String),
    #[error("ObjReaderFactory error: {0}")]
    ObjReaderFactory(String),
    #[error("ObjReader error: {0}")]
    ObjReader(String),
    #[error("File Path Extractor Factory error: {0}")]
    FilePathExtractorFactory(String),
    #[error("File Path Extractor error: {0}")]
    FilePathExtractor(String),
    #[error("FeatureCreator Factory error: {0}")]
    FeatureCreatorFactory(String),
    #[error("FeatureCreator error: {0}")]
    FeatureCreator(String),
    #[error("SqlReader Factory error: {0}")]
    SqlReaderFactory(String),
    #[error("SqlReader error: {0}")]
    SqlReader(String),
    #[error("Geometry parsing error: {0}")]
    GeometryParsing(#[from] GeometryParsingError),
    #[error("Shapefile processing error: {0}")]
    ShapefileProcessing(#[from] ShapefileError),
}

#[derive(Error, Debug)]
pub enum ShapefileError {
    #[error("Direct shapefile bytes not supported. Please provide a ZIP archive containing the shapefile components (.shp, .dbf, .shx)")]
    DirectBytesNotSupported,
    #[error("UTF-16 encoding is not supported. DBF files with UTF-16 require different byte-level decoding. Please convert the shapefile to UTF-8 encoding using a tool like ogr2ogr: ogr2ogr -f \"ESRI Shapefile\" output.shp input.shp -lco ENCODING=UTF-8")]
    Utf16NotSupported,
    #[error("Unsupported encoding: {0}. Supported encodings include: UTF-8, Windows-1250 through Windows-1258, ISO-8859-1 through ISO-8859-16, Shift-JIS, EUC-JP, EUC-KR, Big5, GBK, GB18030, KOI8-R, KOI8-U, IBM866, Macintosh, and others")]
    UnsupportedEncoding(String),
    #[error("Unsupported shape type: {0}")]
    UnsupportedShapeType(String),
    #[error("Polygon has no rings")]
    PolygonNoRings,
    #[error("Polygon has no outer rings")]
    PolygonNoOuterRings,
}

#[derive(Error, Debug)]
pub enum GeometryParsingError {
    #[error("Column not found: {0}")]
    ColumnNotFound(String),
    #[error("Invalid coordinate in column '{column}': {value}")]
    InvalidCoordinate { column: String, value: String },
    #[error("Failed to parse WKT: {0}")]
    WktParsing(String),
    #[error("Failed to convert WKT to geometry: {0}")]
    WktConversion(String),
    #[error("GeometryCollection is not yet supported in CSV reader")]
    UnsupportedGeometryCollection,
    #[error("Unsupported geometry type: {0}")]
    UnsupportedGeometryType(String),
}

pub type Result<T, E = SourceError> = std::result::Result<T, E>;

impl SourceError {
    pub(crate) fn file_path_extractor<T: ToString>(message: T) -> Self {
        Self::FilePathExtractor(message.to_string())
    }

    pub(crate) fn sql_reader<T: ToString>(message: T) -> Self {
        Self::SqlReader(message.to_string())
    }

    pub(crate) fn shapefile_reader<T: ToString>(message: T) -> Self {
        Self::ShapefileReader(message.to_string())
    }
}
