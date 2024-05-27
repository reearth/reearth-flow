use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub(super) enum GeometryProcessorError {
    #[error("Extruder Factory error: {0}")]
    ExtruderFactory(String),
    #[error("Extruder error: {0}")]
    Extruder(String),
    #[error("ThreeDimentionBoxReplacer Factory error: {0}")]
    ThreeDimentionBoxReplacerFactory(String),
    #[error("ThreeDimentionBoxReplacer error: {0}")]
    ThreeDimentionBoxReplacer(String),
    #[error("CoordinateSystemSetter Factory error: {0}")]
    CoordinateSystemSetterFactory(String),
    #[error("CoordinateSystemSetter error: {0}")]
    CoordinateSystemSetter(String),
}

pub(super) type Result<T, E = GeometryProcessorError> = std::result::Result<T, E>;
