use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

pub type PaintResult<T> = Result<T, PaintError>;

#[derive(Error, Debug)]
pub enum PaintError {
    #[error("internal error")]
    Internal(#[from] InternalError),
    #[error("invalid png: {0}")]
    InvalidPNG(String),
    #[error("png decode error")]
    PNGDecodeError(#[from] png::DecodingError),
    #[error("invalid data: {0}")]
    InvalidData(String),
}

impl ResponseError for PaintError {
    fn error_response(&self) -> HttpResponse {
        use PaintError::*;
        match self {
            Internal(_) => HttpResponse::InternalServerError().into(),
            InvalidPNG(_) | PNGDecodeError(_) | InvalidData(_) => {
                HttpResponse::UnprocessableEntity().into()
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
}

impl ResponseError for InternalError {}
