use actix_web::{http::StatusCode, ResponseError};
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
    fn status_code(&self) -> StatusCode {
        use UserError::*;
        match self {
            UserAlreadyExist => StatusCode::CONFLICT,
            InvalidData(_) => StatusCode::UNPROCESSABLE_ENTITY,
            LoginFailed | NoToken | BadToken => StatusCode::UNAUTHORIZED,
        }
    }
}
