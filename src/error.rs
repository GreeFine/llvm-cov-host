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
    #[error("llvm-cov-pretty failed")]
    LlvmCovPretty,
    #[error("report data is empty")]
    NoReportData,
    #[error("didn't find a source file in the report")]
    NoProjectFile,
    #[error("didn't succeed in finding report filepath with our local repository")]
    FailedReportFilePathReplace,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        error!("{self:#?}");
        match self {
            Self::SerdeError(_) | Self::NoReportData | Self::NoProjectFile => {
                StatusCode::BAD_REQUEST
            }
            Self::IoError(_) | Self::LlvmCovPretty | Self::FailedReportFilePathReplace => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::Git(e) => match e.class() {
                git2::ErrorClass::Ssh => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }
}
