use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum LibError {
    #[error("Not a VGM file - {path}")]
    NotVgmFile { path: String },

    #[error("Invalid data provided to GD3 parser")]
    InvalidInputGd3Parser,

    #[error("Unsupported GD3 version")]
    UnsupportedGd3Version,

    #[error("Failed to parse GD3 data")]
    FailedParseGd3,
}
