//! The errors that stop processing an input feature or checking its city GML,
//! by the module they come from.

use super::{code_value, extents, processor};
use crate::common::errors::PlateauProcessorError;

#[derive(thiserror::Error, Debug)]
pub(super) enum Error {
    #[error(transparent)]
    Processor(#[from] processor::Error),
    #[error(transparent)]
    CodeValue(#[from] code_value::Error),
    #[error(transparent)]
    Extents(#[from] extents::Error),
}

pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

impl From<Error> for PlateauProcessorError {
    fn from(error: Error) -> Self {
        Self::DomainOfDefinitionValidator(error.to_string())
    }
}
