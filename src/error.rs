use std::io;

use actix_web::{http::StatusCode, ResponseError};
use log::error;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Deserialization error: {0:?}")]
    SerdeError(#[from] serde_json::Error),
    #[error("io error: {0:?}")]
    IoError(#[from] io::Error),
    #[error("git error: {0:?}")]
    Git(#[from] git2::Error),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        error!("{self:#?}");
        match self {
            ApiError::SerdeError(_) => StatusCode::BAD_REQUEST,
            ApiError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Git(e) => match dbg!(e.class()) {
                git2::ErrorClass::Ssh => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }
}
