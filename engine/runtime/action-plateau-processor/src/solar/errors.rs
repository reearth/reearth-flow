use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolarPositionError {
    #[error("SolarPositionCalculator Factory error: {0}")]
    Factory(String),
    #[error("SolarPositionCalculator error: {0}")]
    Process(String),
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),
    #[error("Failed to parse time string: {0}")]
    TimeParse(String),
    #[error("Coordinate reprojection failed: {0}")]
    Reprojection(String),
}

#[derive(Error, Debug)]
pub enum CityGmlAttributeInserterError {
    #[error("CityGmlAttributeInserter Factory error: {0}")]
    Factory(String),
    #[error("CityGmlAttributeInserter error: {0}")]
    Process(String),
}
