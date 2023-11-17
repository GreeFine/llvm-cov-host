use std::io;

use actix_web::{http::StatusCode, ResponseError};

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Deserialization error: {0:?}")]
    SerdeError(#[from] serde_json::Error),
    #[error("io Error: {0:?}")]
    IoError(#[from] io::Error),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::SerdeError(_) => StatusCode::BAD_REQUEST,
            ApiError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
