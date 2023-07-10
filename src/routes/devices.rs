use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use govee_rs::{
    models::{DeviceState, Devices, PowerState},
    Color,
};
use serde::Deserialize;

use crate::{
    controllers::DeviceController,
    error::{Error, Result},
    server::AppState,
};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/devices", get(devices))
        .route("/devices/", get(devices))
        .route("/devices/:device", get(device))
        .route("/devices/:device/", get(device))
        .route("/devices/:device/toggle", put(toggle_device))
        .route("/devices/:device/color", put(color_device))
        .with_state(state)
}

// Return the list of devices
async fn devices(State(device_controller): State<DeviceController>) -> Result<Json<Devices>> {
    Ok(Json(device_controller.devices().await?))
}

// Get the state of the specified device
async fn device(
    State(device_controller): State<DeviceController>,
    Path(device): Path<String>,
) -> Result<Json<DeviceState>> {
    Ok(Json(device_controller.state(&device).await?))
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
struct ToggleArgs {
    state: PowerState,
}

// Set the power state of the specified device.
async fn toggle_device(
    State(device_controller): State<DeviceController>,
    Path(device): Path<String>,
    Json(toggle_args): Json<ToggleArgs>,
) -> Result<()> {
    device_controller.toggle(&device, toggle_args.state).await
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
struct ColorArgs {
    color: String,
}

// Set the color of the specified device.
async fn color_device(
    State(device_controller): State<DeviceController>,
    Path(device): Path<String>,
    Json(color_args): Json<ColorArgs>,
) -> Result<()> {
    let color =
        Color::parse(&color_args.color).map_err(|_| Error::InvalidColor(color_args.color))?;

    device_controller.color(&device, color).await
}
