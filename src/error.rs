use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("QR code error")]
    QrCode,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "A database error occurred"),
            AppError::QrCode => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate QR code"),
        };

        (status, message).into_response()
    }
}