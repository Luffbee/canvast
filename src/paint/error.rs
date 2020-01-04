use actix_web::{http::StatusCode, ResponseError};
use thiserror::Error;
use tokio::sync::watch::error::SendError;

pub type PaintResult<T> = Result<T, PaintError>;

#[derive(Error, Debug)]
pub enum PaintError {
    #[error("internal error")]
    Internal(#[from] InternalError),
    #[error("invalid png name")]
    InvalidPNGName,
    #[error("invalid png: {0}")]
    InvalidPNG(String),
    #[error("png decode error")]
    PNGDecodeError(#[from] png::DecodingError),
    #[error("invalid data: {0}")]
    InvalidData(String),
}

impl ResponseError for PaintError {
    fn status_code(&self) -> StatusCode {
        use PaintError::*;
        match self {
            Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            InvalidPNGName | InvalidPNG(_) | PNGDecodeError(_) | InvalidData(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum InternalError {
    #[error("png encode error")]
    PNGEncodeError(#[from] png::EncodingError),
    #[error("zip error")]
    ZipError(#[from] zip::result::ZipError),
    #[error("block loading broadcast error")]
    BlockLoadSendError(#[from] SendError<()>),
    #[error("block loading retry limit exceeded")]
    BlockLoadLimitExceeded,
}

impl ResponseError for InternalError {}
