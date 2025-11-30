use crate::adapters;
use crate::adapters::http::app_state::AppState;
use crate::infrastructure::setup::init_tracing;
use axum::{http, Router};
use tower_http::cors::{Any, CorsLayer};

pub fn create_app(app_state: AppState) -> Router {
    init_tracing();

    let cors = CorsLayer::new()
        .allow_origin([
            "https://app.catalytics.pro".parse().unwrap(),
            "https://staging.app.catalytics.pro".parse().unwrap(),
            "http://localhost:4200".parse().unwrap(),
        ])
        .allow_methods([http::Method::GET, http::Method::POST, http::Method::PATCH])
        .allow_headers(Any);

    Router::new()
        .nest("/api", adapters::http::routes::router())
        .with_state(app_state)
        .layer(cors)
}
