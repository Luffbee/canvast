use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

pub type UserResult<T> = Result<T, UserError>;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("user already exist")]
    UserAlreadyExist,
    #[error("invalid data: {0}")]
    InvalidData(String),
    #[error("username and password not match")]
    LoginFailed,
    #[error("no token")]
    NoToken,
    #[error("invalid token")]
    BadToken,
}

impl ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        use UserError::*;
        match self {
            UserAlreadyExist => HttpResponse::Conflict().into(),
            InvalidData(_) => HttpResponse::UnprocessableEntity().into(),
            LoginFailed | NoToken | BadToken => HttpResponse::Unauthorized().into(),
        }
    }
}
