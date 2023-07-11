use anyhow::Result;
use axum::{extract::FromRef, Router};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    controllers::DeviceController,
    routes::{devices, health},
    settings::Settings,
};

#[derive(Clone, FromRef)]
pub struct AppState {
    settings: Settings,
    device_controller: DeviceController,
}

pub async fn serve(settings: Settings) -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "govee_web=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let socket_addr = settings.socket_addr();
    let device_controller = DeviceController::try_from(&settings)?;
    let state = AppState {
        settings,
        device_controller,
    };

    tracing::debug!("listening on {}", &socket_addr);

    axum::Server::bind(&socket_addr)
        .serve(router(state).into_make_service())
        .await?;

    Ok(())
}

fn router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", devices::routes(state.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        // define this after the tracing layer so we don't get spans for the
        // health check endpoint
        .merge(health::routes(state))
}
