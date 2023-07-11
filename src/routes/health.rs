use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use clap::crate_version;
use serde::Serialize;

use crate::{controllers::DeviceController, server::AppState};

const VERSION: &str = crate_version!();

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(state)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HealthStatus {
    Ok,
    Degraded,
    Error,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub struct Health {
    version: &'static str,
    status: HealthStatus,
}

async fn health(State(device_controller): State<DeviceController>) -> impl IntoResponse {
    let mut h = Health {
        version: VERSION,
        status: HealthStatus::Ok,
    };

    if device_controller.health_check().await.is_ok() {
        (StatusCode::OK, Json(h))
    } else {
        h.status = HealthStatus::Error;
        (StatusCode::INTERNAL_SERVER_ERROR, Json(h))
    }
}
