use axum::{http::StatusCode, response::IntoResponse};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid color: '{0}'")]
    InvalidColor(String),

    #[error("Unknown device '{0}'")]
    DeviceNotFound(String),

    #[error(transparent)]
    GoveeClient(#[from] govee_rs::client::GoveeError),

    #[error(transparent)]
    Unhandled(#[from] anyhow::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InvalidColor(ref color) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid color '{}'", color),
            )
                .into_response(),
            Self::DeviceNotFound(ref device) => (
                StatusCode::NOT_FOUND,
                format!("Unknown device '{}'", device),
            )
                .into_response(),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_SERVER_ERROR").into_response(),
        }
    }
}
