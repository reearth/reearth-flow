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
    #[error("GeometryFilter Factory error: {0}")]
    GeometryFilterFactory(String),
    #[error("GeometryFilter error: {0}")]
    GeometryFilter(String),
    #[error("Reprojector Factory error: {0}")]
    ReprojectorFactory(String),
    #[error("Reprojector error: {0}")]
    Reprojector(String),
    #[error("TwoDimentionForcer Factory error: {0}")]
    TwoDimentionForcerFactory(String),
    #[error("TwoDimentionForcer error: {0}")]
    TwoDimentionForcer(String),
}

pub(super) type Result<T, E = GeometryProcessorError> = std::result::Result<T, E>;
