use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NssaCoreError {
    #[error("Invalid transaction: {0}")]
    DeserializationError(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
